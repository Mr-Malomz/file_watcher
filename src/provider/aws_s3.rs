/// Uploads a file to an Amazon S3 bucket.
///
/// # Parameters
///
/// - `bucket_name`: The name of the S3 bucket where the file will be uploaded.
/// - `path_to_file`: The local file path of the file to be uploaded.
/// - `key`: The key (or object name) under which the file will be stored in the S3 bucket.
/// - `region`: The AWS region where the S3 bucket is located.
///
/// # Returns
///
/// Returns a `Result` containing the `PutObjectOutput` on success, or an `SdkError<PutObjectError>`
/// on failure.
///
/// # Errors
///
/// This function may return an error in the following cases:
/// - If the file at `path_to_file` cannot be read or converted into a `ByteStream`.
/// - If the S3 client fails to upload the file due to network issues, invalid credentials,
///   or other AWS SDK-related errors.
///
/// # Example
///
/// ```rust
/// let result = upload_file_to_s3("my-bucket", "/path/to/file.txt", "file.txt", "us-east-1").await;
/// match result {
///     Ok(output) => println!("File uploaded successfully: {:?}", output),
///     Err(e) => eprintln!("Failed to upload file: {:?}", e),
/// }
/// ```
///
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::operation::put_object::{PutObjectError, PutObjectOutput};
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::{Client, config::Region, error::SdkError};

pub async fn upload_file_to_s3(
    bucket_name: &str,
    path_to_file: &str,
    key: &str,
    region: &str,
) -> Result<PutObjectOutput, SdkError<PutObjectError>> {
    // Create a region provider chain
    let region_provider =
        RegionProviderChain::first_try(Region::new(region.to_string())).or_default_provider();

    // Create an S3 client
    let shared_config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&shared_config);

    // Create the ByteStream from file and handle the error properly
    let body = ByteStream::from_path(path_to_file)
        .await
        .map_err(|e| SdkError::construction_failure(e))?;

    // Upload the file
    client
        .put_object()
        .bucket(bucket_name)
        .key(key)
        .body(body)
        .send()
        .await
}
