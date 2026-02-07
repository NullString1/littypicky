use crate::{
    config::ImageConfig,
    error::{AppError, Result},
};
use base64::{engine::general_purpose, Engine};
use image::{imageops::FilterType, DynamicImage, GenericImageView};

#[derive(Clone)]
pub struct ImageService {
    config: ImageConfig,
}

impl ImageService {
    pub fn new(config: ImageConfig) -> Self {
        Self { config }
    }

    /// Process image: decode base64, validate, resize, convert to WebP, re-encode to base64
    pub fn process_image(&self, base64_input: &str) -> Result<String> {
        // Validate base64 format first
        self.validate_base64(base64_input)?;

        // Remove data URI prefix if present
        let base64_data = if base64_input.contains("base64,") {
            base64_input.split("base64,").nth(1).unwrap() // Safe because validate_base64 already checked this
        } else {
            base64_input
        };

        // Decode base64
        let image_data = general_purpose::STANDARD.decode(base64_data).unwrap(); // Safe because validate_base64 already decoded it

        // Check size limit
        let max_size_bytes = self.config.max_size_mb * 1024 * 1024;
        if image_data.len() > max_size_bytes {
            return Err(AppError::Image(format!(
                "Image size exceeds {}MB limit",
                self.config.max_size_mb
            )));
        }

        // Load image
        let img = image::load_from_memory(&image_data)
            .map_err(|e| AppError::Image(format!("Failed to load image: {}", e)))?;

        // Validate dimensions
        let (width, height) = img.dimensions();
        if width == 0 || height == 0 {
            return Err(AppError::Image("Invalid image dimensions".to_string()));
        }
        if width > 10000 || height > 10000 {
            return Err(AppError::Image(
                "Image dimensions too large (max 10000x10000)".to_string(),
            ));
        }

        // Resize if necessary
        let resized_img = self.resize_image(img);

        // Convert to WebP
        let webp_data = self.convert_to_webp(&resized_img)?;

        // Encode back to base64
        let encoded = general_purpose::STANDARD.encode(&webp_data);

        Ok(encoded)
    }

    fn resize_image(&self, img: DynamicImage) -> DynamicImage {
        let (width, height) = img.dimensions();

        if width <= self.config.max_width && height <= self.config.max_height {
            return img;
        }

        let aspect_ratio = width as f32 / height as f32;

        let (new_width, new_height) = if width > height {
            (
                self.config.max_width,
                (self.config.max_width as f32 / aspect_ratio) as u32,
            )
        } else {
            (
                (self.config.max_height as f32 * aspect_ratio) as u32,
                self.config.max_height,
            )
        };

        img.resize(new_width, new_height, FilterType::Lanczos3)
    }

    fn convert_to_webp(&self, img: &DynamicImage) -> Result<Vec<u8>> {
        // Convert to RGB8 for WebP encoding
        let rgb_img = img.to_rgb8();

        // Create WebP encoder
        let encoder = webp::Encoder::from_rgb(rgb_img.as_raw(), img.width(), img.height());

        // Encode with configured quality
        let webp_memory = encoder.encode(self.config.webp_quality);

        Ok(webp_memory.to_vec())
    }

    /// Validate that input is valid base64 (doesn't process, just validates)
    pub fn validate_base64(&self, base64_input: &str) -> Result<()> {
        let base64_data = if base64_input.contains("base64,") {
            base64_input
                .split("base64,")
                .nth(1)
                .ok_or_else(|| AppError::Image("Invalid base64 format".to_string()))?
        } else {
            base64_input
        };

        general_purpose::STANDARD
            .decode(base64_data)
            .map_err(|e| AppError::Image(format!("Invalid base64: {}", e)))?;

        Ok(())
    }
}
