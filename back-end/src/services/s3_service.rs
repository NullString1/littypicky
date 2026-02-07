use crate::error::{AppError, Result};
use aws_config::BehaviorVersion;
use aws_sdk_s3::{
    config::{Credentials, Region},
    primitives::ByteStream,
    Client,
};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct S3Config {
    pub endpoint: String,
    pub region: String,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
    pub public_url: String,
}

#[derive(Clone)]
pub struct S3Service {
    client: Arc<Client>,
    config: S3Config,
}

impl S3Service {
    /// Create a new S3 service
    pub async fn new(config: S3Config) -> Result<Self> {
        // Create credentials
        let credentials = Credentials::new(
            &config.access_key,
            &config.secret_key,
            None,
            None,
            "static",
        );

        // Build S3 config with custom endpoint (for MinIO)
        let s3_config = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new(config.region.clone()))
            .credentials_provider(credentials)
            .endpoint_url(&config.endpoint)
            .load()
            .await;

        let client = Client::new(&s3_config);

        Ok(Self {
            client: Arc::new(client),
            config,
        })
    }

    /// Initialize the S3 bucket (create if doesn't exist)
    pub async fn initialize(&self) -> Result<()> {
        // Check if bucket exists
        let bucket_exists = self
            .client
            .head_bucket()
            .bucket(&self.config.bucket)
            .send()
            .await
            .is_ok();

        if !bucket_exists {
            tracing::info!("Creating S3 bucket: {}", self.config.bucket);
            self.client
                .create_bucket()
                .bucket(&self.config.bucket)
                .send()
                .await
                .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to create bucket: {}", e)))?;

            // Set bucket policy to allow public read access to images
            let policy = format!(
                r#"{{
                    "Version": "2012-10-17",
                    "Statement": [
                        {{
                            "Effect": "Allow",
                            "Principal": {{"AWS": ["*"]}},
                            "Action": ["s3:GetObject"],
                            "Resource": ["arn:aws:s3:::{}/*"]
                        }}
                    ]
                }}"#,
                self.config.bucket
            );

            self.client
                .put_bucket_policy()
                .bucket(&self.config.bucket)
                .policy(policy)
                .send()
                .await
                .map_err(|e| {
                    AppError::Internal(anyhow::anyhow!("Failed to set bucket policy: {}", e))
                })?;

            tracing::info!("Bucket created and configured successfully");
        } else {
            tracing::info!("S3 bucket already exists: {}", self.config.bucket);
        }

        Ok(())
    }

    /// Upload image to S3 and return the public URL
    /// Takes processed WebP image data
    pub async fn upload_image(&self, image_data: Vec<u8>, prefix: &str) -> Result<String> {
        // Generate unique filename
        let filename = format!("{}/{}.webp", prefix, Uuid::new_v4());

        // Upload to S3
        self.client
            .put_object()
            .bucket(&self.config.bucket)
            .key(&filename)
            .body(ByteStream::from(image_data))
            .content_type("image/webp")
            .send()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to upload to S3: {}", e)))?;

        // Return public URL
        let url = format!("{}/{}", self.config.public_url, filename);
        Ok(url)
    }

    /// Get image data from S3
    pub async fn get_image(&self, key: &str) -> Result<Vec<u8>> {
        let response = self
            .client
            .get_object()
            .bucket(&self.config.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| {
                if e.to_string().contains("NoSuchKey") {
                    AppError::NotFound("Image not found".to_string())
                } else {
                    AppError::Internal(anyhow::anyhow!("Failed to get from S3: {}", e))
                }
            })?;

        let data = response
            .body
            .collect()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to read S3 response: {}", e)))?;

        Ok(data.into_bytes().to_vec())
    }

    /// Delete image from S3
    pub async fn delete_image(&self, key: &str) -> Result<()> {
        self.client
            .delete_object()
            .bucket(&self.config.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to delete from S3: {}", e)))?;

        Ok(())
    }

    /// Extract S3 key from public URL
    pub fn extract_key_from_url(&self, url: &str) -> Option<String> {
        url.strip_prefix(&format!("{}/", self.config.public_url))
            .map(String::from)
    }
}
