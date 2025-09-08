use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::{Client, config::Region};
use crate::command::data::AwsUploadResponse;
use std::path::Path;

pub async fn upload_file_to_s3(
    bucket_name: &str,
    path_to_file: &str,
    key: &str,
    region: &str,
) -> Result<AwsUploadResponse, Box<dyn std::error::Error>> {
    // Create a region provider chain
    let region_provider =
        RegionProviderChain::first_try(Region::new(region.to_string())).or_default_provider();

    // Create an S3 client
    let shared_config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&shared_config);

    // Extract filename from path
    let file_path = Path::new(path_to_file);
    let filename = file_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("Invalid file path")?;

    // Create the ByteStream from file and handle the error properly
    let body = ByteStream::from_path(path_to_file)
        .await
        .map_err(|e| format!("Failed to read file: {}", e))?;

    // Upload the file
    let result = client
        .put_object()
        .bucket(bucket_name)
        .key(key)
        .body(body)
        .send()
        .await;

    match result {
        Ok(_output) => {
            println!("File '{}' uploaded successfully to S3 bucket: {}", filename, bucket_name);
            Ok(AwsUploadResponse {
                success: true,
                message: format!("Successfully uploaded {} to s3://{}/{}", filename, bucket_name, key),
            })
        }
        Err(e) => {
            Err(format!("AWS S3 upload error: {}", e).into())
        }
    }
}
