use std::io::Cursor;

use image::ImageReader;

use crate::domain::assets::ImageAsset;

/// Maintains parity across native and web builds
///
/// For more information see `maxTextureDimension2D` in WebGPU
const MAX_IMAGE_DIMENSION: u32 = 8192;

/// Decodes raw image bytes into an `ImageAsset`.
/// Preserves original bytes so the source image remains intact in I/O
pub fn decode_image_asset(bytes: Vec<u8>, name: String) -> Result<ImageAsset, String> {
    let reader = ImageReader::new(Cursor::new(&bytes))
        .with_guessed_format()
        .map_err(|err| err.to_string())?;

    let extension = reader
        .format()
        .and_then(|format| format.extensions_str().first())
        .copied()
        .unwrap_or("png")
        .to_string();

    let mut image = reader.decode().map_err(|err| err.to_string())?;

    if image.width() > MAX_IMAGE_DIMENSION || image.height() > MAX_IMAGE_DIMENSION {
        let scale = (MAX_IMAGE_DIMENSION as f32 / image.width() as f32)
            .min(MAX_IMAGE_DIMENSION as f32 / image.height() as f32);

        let new_width = ((image.width() as f32 * scale).floor() as u32).max(1);
        let new_height = ((image.height() as f32 * scale).floor() as u32).max(1);

        image = image.resize(new_width, new_height, image::imageops::FilterType::Lanczos3);
    }

    let width = image.width();
    let height = image.height();

    Ok(ImageAsset {
        encoded: bytes,
        extension,
        data: image.into_bytes(),
        width,
        height,
        name,
    })
}
