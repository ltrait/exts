use ltrait::{source::Source, tokio_stream};

pub mod icon;

#[derive(Debug, thiserror::Error)]
pub enum DesktopError {
    // Stringは名前
    #[error("Failed to find icon of {0}")]
    NoIcon(String),
    #[error("Failed to open file: {0}")]
    OpenFile(#[source] std::io::Error),
    #[error("Failed to decode the image: {0}")]
    ImageDecode(#[source] image::ImageError),
}

pub struct DesktopEntry {
    pub entry: freedesktop_desktop_entry::DesktopEntry,

    /// base64 encoded icon of the entry
    pub icon: Option<String>,
}

// 楽をするためにfreedesktop_desktop_entryを使っているからStreamではなく、性能を最大限に活かしきれていない
pub fn new<'a>() -> Result<Source<'a, DesktopEntry>, DesktopError> {
    use freedesktop_desktop_entry::{Iter, default_paths};
    let entries = Iter::new(default_paths()).entries::<String>(None).map(|e| {
        let icon = e.icon().and_then(|n| icon::lookup(n).ok());
        DesktopEntry { entry: e, icon }
    });

    Ok(Box::pin(tokio_stream::iter(entries)))
}
