// Shared S3/MinIO helper functions.

use s3::bucket::Bucket;
use s3::creds::Credentials;
use s3::Region;

use crate::config::Config;

/// Build a MinIO Bucket instance from the application config.
fn build_bucket(config: &Config) -> anyhow::Result<Box<Bucket>> {
    let credentials = Credentials::new(
        Some(&config.minio_access_key),
        Some(&config.minio_secret_key),
        None,
        None,
        None,
    )?;

    let region = Region::Custom {
        region: "us-east-1".to_string(),
        endpoint: config.minio_endpoint.clone(),
    };

    let bucket = Bucket::new(&config.minio_bucket, region, credentials)?
        .with_path_style(); // MinIO requires path-style addressing

    Ok(bucket)
}

/// Upload data to MinIO at the given object key.
pub async fn upload_object(
    config: &Config,
    key: &str,
    data: &[u8],
    content_type: &str,
) -> anyhow::Result<()> {
    let bucket = build_bucket(config)?;
    bucket
        .put_object_with_content_type(key, data, content_type)
        .await?;
    Ok(())
}

/// Generate a presigned GET URL for a MinIO object (5-minute expiry).
pub async fn presign_get_url(config: &Config, object_key: &str) -> anyhow::Result<String> {
    let bucket = build_bucket(config)?;
    let url = bucket.presign_get(object_key, 300, None).await?;
    Ok(url)
}
