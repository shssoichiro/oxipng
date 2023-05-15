use image::{DynamicImage, GenericImageView, ImageFormat, Pixel};
use log::{error, warn};
use std::io::Cursor;

/// Validate that the output png data still matches the original image
pub fn validate_output(output: &[u8], original_data: &[u8]) -> bool {
    let (old_png, new_png) = rayon::join(
        || load_png_image_from_memory(original_data),
        || load_png_image_from_memory(output),
    );

    match (new_png, old_png) {
        (Err(new_err), _) => {
            error!("Failed to read output image for validation: {}", new_err);
            false
        }
        (_, Err(old_err)) => {
            // The original image might be invalid if, for example, there is a CRC error,
            // and we set fix_errors to true. In that case, all we can do is check that the
            // new image is decodable.
            warn!("Failed to read input image for validation: {}", old_err);
            true
        }
        (Ok(new_png), Ok(old_png)) => images_equal(&old_png, &new_png),
    }
}

/// Loads a PNG image from memory to a [DynamicImage]
fn load_png_image_from_memory(png_data: &[u8]) -> Result<DynamicImage, image::ImageError> {
    let mut reader = image::io::Reader::new(Cursor::new(png_data));
    reader.set_format(ImageFormat::Png);
    reader.no_limits();
    reader.decode()
}

/// Compares images pixel by pixel for equivalent content
fn images_equal(old_png: &DynamicImage, new_png: &DynamicImage) -> bool {
    let a = old_png.pixels().filter(|x| {
        let p = x.2.channels();
        !(p.len() == 4 && p[3] == 0)
    });
    let b = new_png.pixels().filter(|x| {
        let p = x.2.channels();
        !(p.len() == 4 && p[3] == 0)
    });
    a.eq(b)
}
