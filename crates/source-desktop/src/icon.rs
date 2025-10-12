use crate::DesktopError;
use std::path::{Path, PathBuf};

pub fn lookup(name: &str) -> Result<PathBuf, DesktopError> {
    freedesktop_icons::lookup(name)
        .find()
        .ok_or_else(|| DesktopError::NoIcon(name.into()))
}

// see also https://github.com/satler-git/sandbox/blob/bcab487f5d9c35e938132e2ed15d3c9db729a6a2/rust/icon-base64/src/main.rs
pub fn load_image(path: &Path) -> Result<String, DesktopError> {
    use base64::{Engine as _, engine::general_purpose};
    use image::{ImageFormat, ImageReader};
    use std::io::Cursor;

    let img = ImageReader::open(path)
        .map_err(DesktopError::OpenFile)?
        .decode()
        .map_err(DesktopError::ImageDecode)?;

    let mut buff = Cursor::new(vec![]);

    img.write_to(&mut buff, ImageFormat::Png)
        .map_err(DesktopError::ImageDecode)?;

    Ok(general_purpose::STANDARD.encode(buff.get_ref()))
}
