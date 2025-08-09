use ltrait::{source::Source, tokio_stream};
use std::path::PathBuf;

pub use freedesktop_desktop_entry::default_paths;

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

#[derive(Debug, Clone)]
pub struct DesktopEntry {
    pub entry: freedesktop_desktop_entry::DesktopEntry,
}

impl DesktopEntry {
    pub fn icon(&self) -> Option<String> {
        self.entry.icon().and_then(|n| icon::lookup(n).ok())
    }
}

// 楽をするためにfreedesktop_desktop_entryを使っているからStreamではなく、性能を最大限に活かしきれていない
pub fn new<'a>(
    paths: impl Iterator<Item = PathBuf>,
) -> Result<Source<'a, DesktopEntry>, DesktopError> {
    use freedesktop_desktop_entry::Iter;
    let entries = Iter::new(paths)
        .entries::<String>(None)
        .map(|e| DesktopEntry { entry: e });

    Ok(Box::pin(tokio_stream::iter(entries)))
}
