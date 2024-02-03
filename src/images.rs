use std::{
    cmp::max,
    fs,
    io::{self, BufWriter},
    path::Path,
    sync::atomic::AtomicBool,
};

use image::{imageops, ImageOutputFormat, RgbaImage};
use log::info;
use thiserror::Error;

use crate::{abort::return_on_flag, saving::SavingSemaphore};

#[derive(Error, Debug)]
pub enum ImagesError {
    #[error("IO error: {0}")]
    IO(#[from] io::Error),

    #[error("Image processing error: {0}")]
    Image(#[from] image::ImageError),
}

fn is_bigger(image: &RgbaImage, max_size: u32) -> bool {
    max(image.width(), image.height()) > max_size
}

fn thumbnail(image: &mut RgbaImage, max_size: u32) -> RgbaImage {
    let (width, height) = image.dimensions();
    let scale = max_size as f32 / max(width, height) as f32;
    let new_width = (width as f32 * scale) as u32;
    let new_height = (height as f32 * scale) as u32;
    imageops::resize(
        image,
        new_width,
        new_height,
        imageops::FilterType::CatmullRom,
    )
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

fn save_image(
    image: &RgbaImage,
    path: &Path,
    jpeg_quality: u8,
    saving: &SavingSemaphore,
) -> Result<(), ImagesError> {
    saving.increment();
    image
        .write_to(
            &mut BufWriter::new(fs::File::create(path).map_err({
                saving.decrement();
                ImagesError::IO
            })?),
            ImageOutputFormat::Jpeg(jpeg_quality),
        )
        .map_err({
            saving.decrement();
            ImagesError::Image
        })?;
    saving.decrement();
    Ok(())
}

pub fn save_bytes_as_image(
    bytes: &[u8],
    path: &Path,
    max_size: u32,
    jpeg_quality: u8,
    stopped: &AtomicBool,
    saving: &SavingSemaphore,
) -> Result<(), ImagesError> {
    return_on_flag!(stopped, || info!("Shutting down..."));
    let mut image = image::load_from_memory(bytes)
        .map_err(ImagesError::Image)?
        .to_rgba8();

    return_on_flag!(stopped, || info!("Shutting down..."));
    if is_bigger(&image, max_size) {
        image = thumbnail(&mut image, max_size);
    }

    return_on_flag!(stopped, || info!("Shutting down..."));
    remove_transparency(&mut image);

    return_on_flag!(stopped, || info!("Shutting down..."));
    save_image(&image, path, jpeg_quality, saving)
}
