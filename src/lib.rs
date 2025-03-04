use ltrait::{
    color_eyre::eyre::{OptionExt, Result, WrapErr},
    launcher::batcher::Batcher,
    tokio_stream::StreamExt as _,
    ui::{Buffer, UI},
};

use crossterm::{
    event::{Event as CEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    DefaultTerminal, Frame, TerminalOptions, Viewport,
    layout::{Constraint, Direction, Layout},
    style::Style,
    widgets::{Block, Borders, Paragraph, Widget},
};
use rustc_hash::FxHashMap;
use tui_input::{Input, backend::crossterm::EventHandler};

pub use ratatui::style;

use futures::FutureExt as _;
use std::sync::Arc;
use tokio::sync::{
    Mutex, RwLock,
    mpsc::{self, Sender},
};

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
    pub title: StyledText,
    pub sub_string: Option<StyledText>,
    pub icon: Option<char>,
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

struct App {
    config: TuiConfig,

    exit: bool,
    // 上が0
    selecting_i: usize,
    input: Arc<Mutex<Input>>,
    buffer: RwLock<Buffer<(TuiEntry, usize)>>,
    tx: Option<Sender<Event>>,
    has_more: bool,
    id_to_index: Arc<RwLock<FxHashMap<usize, usize>>>,
    index_to_id: Arc<RwLock<FxHashMap<usize, usize>>>,
}

impl App {
    fn new(config: TuiConfig) -> Self {
        Self {
            config,
            has_more: false,
            exit: false,
            selecting_i: 0,
            input: Mutex::new(Input::default()).into(),
            buffer: RwLock::new(Buffer::default()).into(),
            tx: None,
            id_to_index: RwLock::new(FxHashMap::default()).into(),
            index_to_id: RwLock::new(FxHashMap::default()).into(),
        }
    }
}

enum Event {
    CEvent(CEvent),
    Refresh,
    Next,
}

impl<'a> App {
    async fn run<Cusion: Send + 'a>(
        &mut self,
        terminal: &mut DefaultTerminal,
        batcher: &mut Batcher<'a, Cusion, TuiEntry>,
    ) -> Result<usize> {
        self.has_more = batcher.marge(&mut *self.buffer.write().await).await?;

        let (tx, mut rx) = mpsc::channel(100);

        {
            let txc = tx.clone();
            tokio::spawn(async move {
                let mut reader = crossterm::event::EventStream::new();

                loop {
                    let crossterm_event = reader.next().fuse();
                    std::thread::sleep(std::time::Duration::from_millis(10));

                    match crossterm_event.await {
                        Some(Ok(evt)) => match evt {
                            CEvent::Key(key) => {
                                if key.kind == KeyEventKind::Press {
                                    txc.send(Event::CEvent(CEvent::Key(key))).await.unwrap();
                                }
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
            });
        }

        self.tx = Some(tx.clone());

        while !self.exit {
            let event = rx.recv().await.unwrap();
            terminal.draw(|frame| self.draw(frame))?;

            self.handle_events(event, batcher)
                .await
                .wrap_err("handle events failed")?;

            if self.has_more {
                tx.send(Event::Next)
                    .await
                    .wrap_err("Failed to send message `Event::Next`")?;
            }
        }

        Ok(self.selecting_i)
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
                self.handle_key_event(key_event).await?
            }
            Event::Refresh => {
                let input = &self.input;
                batcher.input(&mut *self.buffer.write().await, input.lock().await.value())
            }
            Event::Next => {
                self.has_more = batcher.marge(&mut *self.buffer.write().await).await?;
            }
            _ => {}
        };
        Ok(())
    }

    async fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match (key_event.code, key_event.modifiers) {
            (KeyCode::Char('c'), KeyModifiers::CONTROL)
            | (KeyCode::Char('d'), KeyModifiers::CONTROL) => self.exit(),
            (KeyCode::Up, _) | (KeyCode::Char('k'), KeyModifiers::CONTROL) => {
                let id_to_index = self.id_to_index.read().await;
                let index = id_to_index.get(&self.selecting_i).unwrap();
                let new_index = (index + 1).max(id_to_index.len() - 1);

                let index_to_id = self.index_to_id.read().await;
                self.selecting_i = *index_to_id.get(&new_index).unwrap();
            }
            (KeyCode::Down, _) | (KeyCode::Char('j'), KeyModifiers::CONTROL) => {
                let id_to_index = self.id_to_index.read().await;
                let index = id_to_index.get(&self.selecting_i).unwrap();
                let new_index = index.saturating_sub(1);

                let index_to_id = self.index_to_id.read().await;
                self.selecting_i = *index_to_id.get(&new_index).unwrap();
            }
            _ => {
                (*self.input)
                    .lock()
                    .await
                    .handle_event(&crossterm::event::Event::Key(key_event))
                    .ok_or_eyre("Failed to lock input")?;
                self.tx
                    .as_mut()
                    .unwrap()
                    .send(Event::Refresh)
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
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
            .split(area);

        {
            let entries_area = chunks[0];
            let mut buf = self.buffer.blocking_write();

            let mut id_to_index = self.id_to_index.blocking_write();
            let mut index_to_id = self.index_to_id.blocking_write();

            let mut index = 0;
            // 残り
            // 選択
            // 2行
            while let Some(&(ref ei, i)) = buf.next() {
                id_to_index.insert(i, index);
                index_to_id.insert(index, i);
                index += 1;
                let index = ();
            }
            buf.reset_pos();
        }
        {
            use tokio::runtime::Runtime;
            let rt = Runtime::new().unwrap();

            let input_text = {
                rt.block_on(async {
                    println!("hello");
                    (*self.input).lock().await.to_string()
                })
            };

            Paragraph::new(input_text)
                .block(Block::default().borders(Borders::ALL))
                .render(chunks[1], buf);
        }
    }
}
