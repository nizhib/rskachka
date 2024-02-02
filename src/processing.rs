use std::{cmp::max, fs, io, io::BufWriter, path::Path, sync::Mutex};

use image::{ImageOutputFormat, RgbaImage};
use thiserror::Error;

use crate::{abort, saving::SavingSemaphore};

#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Image processing error: {0}")]
    Image(#[from] image::ImageError),
}

fn thumbnail(img: &RgbaImage, max_size: u32) -> RgbaImage {
    let new_width = if img.width() > img.height() {
        max_size
    } else {
        (img.width() as f32 * (max_size as f32 / img.height() as f32)) as u32
    };
    let new_height = if img.height() > img.width() {
        max_size
    } else {
        (img.height() as f32 * (max_size as f32 / img.width() as f32)) as u32
    };
    image::imageops::thumbnail(img, new_width, new_height)
}

fn remove_transparency(image: &mut RgbaImage) {
    image.pixels_mut().for_each(|pixel| {
        let alpha = pixel[3] as f32 / 255.0;
        let not_alpha = 255.0 * (1.0 - alpha);
        pixel[0] = (pixel[0] as f32 * alpha + not_alpha) as u8;
        pixel[1] = (pixel[1] as f32 * alpha + not_alpha) as u8;
        pixel[2] = (pixel[2] as f32 * alpha + not_alpha) as u8;
    });
}

fn save_image(image: &RgbaImage, path: &Path, jpeg_quality: u8) -> Result<(), ProcessingError> {
    let file = fs::File::create(path).map_err(ProcessingError::Io)?;

    let mut writer = BufWriter::new(file);
    image
        .write_to(&mut writer, ImageOutputFormat::Jpeg(jpeg_quality))
        .map_err(ProcessingError::Image)
}

pub fn save_bytes_as_image(
    bytes: &[u8],
    path: &Path,
    max_size: u32,
    jpeg_quality: u8,
    stopped: &Mutex<bool>,
    saving: &SavingSemaphore,
) -> Result<(), ProcessingError> {
    abort::return_on_flag!(stopped, "Stopping the worker...");
    let mut image = image::load_from_memory(bytes)
        .map_err(ProcessingError::Image)?
        .to_rgba8();

    abort::return_on_flag!(stopped, "Stopping the worker...");
    remove_transparency(&mut image);
    if max(image.width(), image.height()) > max_size {
        image = thumbnail(&image, max_size);
    }

    abort::return_on_flag!(stopped, "Stopping the worker...");
    saving.increment();
    let saved = save_image(&image, path, jpeg_quality);
    saving.decrement();

    saved
}
