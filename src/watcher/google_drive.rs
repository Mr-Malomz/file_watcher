use crate::command::data::{GoogleDriveListResponse, GoogleDriveFile};
use serde_json::Value;

pub async fn list_google_drive_files(
    access_token: &str,
    max_results: u32,
    query: &str,
) -> Result<GoogleDriveListResponse, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    
    // Build the request URL with parameters
    let mut url = format!(
        "https://www.googleapis.com/drive/v3/files?pageSize={}&fields=files(id,name,size,modifiedTime,mimeType,webViewLink)",
        max_results
    );
    
    if !query.is_empty() {
        url.push_str(&format!("&q={}", urlencoding::encode(query)));
    }
    
    // Send the request
    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await?;
    
    if response.status().is_success() {
        let json: Value = response.json().await?;
        
        let files_array = json["files"].as_array()
            .ok_or("Invalid response: missing 'files' array")?;
        
        let files: Vec<GoogleDriveFile> = files_array
            .iter()
            .map(|file| GoogleDriveFile {
                id: file["id"].as_str().unwrap_or("").to_string(),
                name: file["name"].as_str().unwrap_or("Untitled").to_string(),
                size: file["size"].as_str().map(|s| s.to_string()),
                modified_time: file["modifiedTime"].as_str().map(|s| s.to_string()),
                mime_type: file["mimeType"].as_str().map(|s| s.to_string()),
                web_view_link: file["webViewLink"].as_str().map(|s| s.to_string()),
            })
            .collect();
        
        let total_count = files.len();
        
        println!("Successfully listed {} files from Google Drive", total_count);
        
        Ok(GoogleDriveListResponse {
            files,
            total_count,
        })
    } else {
        let error_text = response.text().await?;
        Err(format!("Google Drive API error: {}", error_text).into())
    }
}
