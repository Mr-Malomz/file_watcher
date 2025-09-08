use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::{Client, config::Region};
use crate::command::data::{AwsListResponse, AwsObject};

pub async fn list_s3_objects(
    bucket_name: &str,
    region: &str,
    prefix: &str,
    max_keys: i32,
) -> Result<AwsListResponse, Box<dyn std::error::Error>> {
    // Create a region provider chain
    let region_provider =
        RegionProviderChain::first_try(Region::new(region.to_string())).or_default_provider();

    // Create an S3 client
    let shared_config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&shared_config);

    // Build the list objects request
    let mut request = client.list_objects_v2().bucket(bucket_name).max_keys(max_keys);
    
    if !prefix.is_empty() {
        request = request.prefix(prefix);
    }

    // Send the request
    let result = request.send().await;

    match result {
        Ok(output) => {
            let objects: Vec<AwsObject> = output
                .contents()
                .iter()
                .map(|obj| AwsObject {
                    key: obj.key().unwrap_or("").to_string(),
                    size: obj.size().unwrap_or(0) as u64,
                    last_modified: obj
                        .last_modified()
                        .map(|dt| dt.to_string())
                        .unwrap_or_else(|| "Unknown".to_string()),
                    storage_class: obj.storage_class().map(|sc| sc.as_str().to_string()),
                })
                .collect();

            let total_count = objects.len();
            
            println!("Successfully listed {} objects from S3 bucket: {}", total_count, bucket_name);
            
            Ok(AwsListResponse {
                objects,
                total_count,
            })
        }
        Err(e) => {
            Err(format!("AWS S3 list error: {}", e).into())
        }
    }
}
