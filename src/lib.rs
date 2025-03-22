use ltrait::{
    color_eyre::eyre::{OptionExt, Result, WrapErr, bail},
    launcher::batcher::Batcher,
    tokio_stream::StreamExt as _,
    ui::{Buffer, Position, UI},
};

use crossterm::{
    event::{Event as CEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    DefaultTerminal, Frame, TerminalOptions, Viewport,
    layout::{Constraint, Direction, Layout},
    style::Style,
    widgets::{Block, Borders, List, Paragraph, Widget},
};
use tracing::{debug, info};
use tui_input::{Input, backend::crossterm::EventHandler};

pub use ratatui::style;

use futures::{FutureExt as _, join, select};
use tokio::sync::mpsc;

pub struct Tui {
    config: TuiConfig,
}

#[derive(Clone)]
pub struct TuiConfig {
    lines: u16,
    selecting: char,
    no_selecting: char,
}

impl TuiConfig {
    pub fn new(lines: u16, selecting: char, no_selecting: char) -> Self {
        Self {
            lines,
            selecting,
            no_selecting,
        }
    }
}

impl Tui {
    pub fn new(config: TuiConfig) -> Self {
        Self { config }
    }
}

type StyledText = (String, Style);

/// `<SelectingStatus> <icon> <title> <sub_string>`
/// SelectingStatus in above is a char
pub struct TuiEntry {
    pub text: StyledText,
}

impl<'a> UI<'a> for Tui {
    type Context = TuiEntry;

    async fn run<Cusion: 'a + Send>(
        &self,
        mut batcher: Batcher<'a, Cusion, Self::Context>,
    ) -> Result<Cusion> {
        let mut terminal = ratatui::init_with_options(TerminalOptions {
            viewport: Viewport::Inline(self.config.lines),
        });

        enable_raw_mode()?;
        terminal.clear()?;

        let i = App::new(self.config.clone())
            .run(&mut terminal, &mut batcher)
            .await?;

        disable_raw_mode()?;
        ratatui::restore();

        batcher.compute_cusion(i)
    }
}

// なんのArc, Mutex, RwLockを使うか検討する必要がある。renderの中で使えないと意味ないし
struct App {
    config: TuiConfig,

    exit: bool,
    // 上が0
    selecting_i: usize,
    input: Input,
    buffer: Buffer<(TuiEntry, usize)>,
    has_more: bool,
    tx: Option<mpsc::Sender<Event>>,
}

impl App {
    fn new(config: TuiConfig) -> Self {
        Self {
            has_more: true,
            config,
            exit: false,
            selecting_i: 0,
            input: Input::default(),
            buffer: Buffer::default(),
            tx: None,
        }
    }
}

#[derive(Debug)]
enum Event {
    CEvent(CEvent),
    Refresh,
    Input,
}

impl Event {
    async fn terminal_event_listener(tx: mpsc::Sender<Event>) {
        let mut reader = crossterm::event::EventStream::new();

        loop {
            let crossterm_event = reader.next().fuse();
            std::thread::sleep(std::time::Duration::from_millis(10));

            if let Some(Ok(evt)) = crossterm_event.await {
                if let CEvent::Key(key) = evt {
                    if key.kind == KeyEventKind::Press {
                        tx.send(Event::CEvent(CEvent::Key(key))).await.unwrap();
                    }
                }
            }
        }
    }
}

impl<'a> App {
    async fn run<Cusion: Send + 'a>(
        &mut self,
        terminal: &mut DefaultTerminal,
        batcher: &mut Batcher<'a, Cusion, TuiEntry>,
    ) -> Result<usize> {
        let (tx, mut rx) = mpsc::channel(100);

        tokio::spawn(Event::terminal_event_listener(tx.clone()));
        self.tx = Some(tx.clone());

        while !self.exit {
            let prepare = async {
                if self.has_more {
                    batcher.prepare().await
                } else {
                    // HACK: もうeventだけ気にしていればいいから
                    info!("No more items. Sleeping");
                    tokio::time::sleep(std::time::Duration::from_secs(100)).await;
                    batcher.prepare().await
                }
            };

            select! {
                // TODO: 毎回futureを生成し直していると
                // dropした場合にバグるかも。あと必ず、rx.recvが早い場合何も表示されなくなっちゃうかも
                from = prepare.fuse() => {
                    info!("Merging");
                    let (has_more, _) = join!(
                        batcher.merge(&mut self.buffer, from),
                        tx.send(Event::Refresh),
                    );

                    self.has_more = has_more?;
                    info!("Merged");
                }
                event_like = rx.recv().fuse() => {
                    info!("Caught event-like");
                    debug!("{event_like:?}");

                    match event_like {
                        Some(event) => {
                            self.handle_events(event, batcher)
                                .await
                                .wrap_err("handle events failed")?;

                            terminal.draw(|frame| self.draw(frame))?;
                        }
                    _ => bail!("the communication channel for event was unexpectedly closed.")
                    }
                }
            }
        }

        Ok({
            let mut pos = Position(self.selecting_i);
            self.buffer.next(&mut pos).unwrap().1
        })
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    async fn handle_events<Cusion: Send + 'a>(
        &mut self,
        event: Event,
        batcher: &mut Batcher<'a, Cusion, TuiEntry>,
    ) -> Result<()> {
        match event {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::CEvent(CEvent::Key(key_event)) if key_event.kind == KeyEventKind::Press => {
                info!("Handling KeyInput");
                self.handle_key_event(key_event).await?
            }
            Event::Input => {
                info!("Handling Input");
                batcher.input(&mut self.buffer, self.input.value());
                // 一回一番上に戻す
                self.selecting_i = 0;
                self.has_more = true;
            }
            _ => {}
        };
        Ok(())
    }

    async fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match (key_event.code, key_event.modifiers) {
            (KeyCode::Enter, _) => {
                self.exit();
            }
            (KeyCode::Char('c'), KeyModifiers::CONTROL)
            | (KeyCode::Char('d'), KeyModifiers::CONTROL) => self.exit(),
            (KeyCode::Up, _) | (KeyCode::Char('k'), KeyModifiers::CONTROL) => {
                self.selecting_i = (self.selecting_i + 1).min(self.buffer.len().saturating_sub(1));
            }
            (KeyCode::Down, _) | (KeyCode::Char('j'), KeyModifiers::CONTROL) => {
                self.selecting_i = self.selecting_i.saturating_sub(1);
            }
            _ => {
                self.input
                    .handle_event(&crossterm::event::Event::Key(key_event))
                    .ok_or_eyre("Failed to lock input")?;

                self.tx
                    .as_mut()
                    .unwrap()
                    .send(Event::Input)
                    .await
                    .wrap_err("Failed to send Refresh")?;
            }
        }
        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: ratatui::prelude::Rect, buffer: &mut ratatui::prelude::Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(2)].as_ref())
            .split(area);

        // エントリーの部分
        {
            let list_area = chunks[0];

            let items_count = self.buffer.len();
            let mut items = Vec::with_capacity(items_count);

            let mut pos = Position::default();

            while let Some((entry, _)) = self.buffer.next(&mut pos) {
                let is_selected = pos.0 - 1 == self.selecting_i;

                let selecting_status = if is_selected {
                    self.config.selecting
                } else {
                    self.config.no_selecting
                };

                let entry_text = format!("{} {}", selecting_status, entry.text.0);
                let style = entry.text.1;

                // リストアイテムを追加
                items.push(ratatui::widgets::ListItem::new(entry_text).style(style));
            }

            items.reverse();

            let visible_height = list_area.height as usize;
            let reversed_selecting_index = items_count - 1 - self.selecting_i;

            // 選択されたアイテムが常に表示されるようにスクロール位置を計算
            let margin_below = 2;
            let scroll_offset = if reversed_selecting_index > visible_height - margin_below - 1 {
                reversed_selecting_index - (visible_height - margin_below - 1)
            } else {
                0
            };

            let start_index = scroll_offset;
            let end_index = (scroll_offset + visible_height).min(items_count);

            let items: Vec<_> = items
                .into_iter()
                .skip(start_index)
                .take(end_index - start_index)
                .collect();

            List::new(items)
                .block(Block::default())
                .render(list_area, buffer);
        }
        // テキスト入力部分
        {
            let input_area = chunks[1];
            let input_text = self.input.to_string();

            Paragraph::new(input_text)
                .block(Block::default().borders(Borders::TOP))
                .render(input_area, buffer);
        }
    }
}
