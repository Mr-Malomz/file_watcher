use crate::command::data::{DropboxListResponse, DropboxEntry};
use serde_json::{json, Value};

pub async fn list_dropbox_files(
    access_token: &str,
    path: &str,
    limit: u32,
) -> Result<DropboxListResponse, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    
    // Prepare the API arguments
    let api_args = json!({
        "path": if path.is_empty() { "" } else { path },
        "recursive": false,
        "include_media_info": false,
        "include_deleted": false,
        "include_has_explicit_shared_members": false,
        "include_mounted_folders": true,
        "limit": limit
    });
    
    // Send the request
    let response = client
        .post("https://api.dropboxapi.com/2/files/list_folder")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Content-Type", "application/json")
        .json(&api_args)
        .send()
        .await?;
    
    if response.status().is_success() {
        let json: Value = response.json().await?;
        
        let entries_array = json["entries"].as_array()
            .ok_or("Invalid response: missing 'entries' array")?;
        
        let entries: Vec<DropboxEntry> = entries_array
            .iter()
            .map(|entry| DropboxEntry {
                name: entry["name"].as_str().unwrap_or("").to_string(),
                path_lower: entry["path_lower"].as_str().unwrap_or("").to_string(),
                path_display: entry["path_display"].as_str().unwrap_or("").to_string(),
                id: entry["id"].as_str().map(|s| s.to_string()),
                size: entry["size"].as_u64(),
                client_modified: entry["client_modified"].as_str().map(|s| s.to_string()),
                server_modified: entry["server_modified"].as_str().map(|s| s.to_string()),
                tag: entry[".tag"].as_str().unwrap_or("unknown").to_string(),
            })
            .collect();
        
        let total_count = entries.len();
        
        println!("Successfully listed {} entries from Dropbox folder: {}", 
                total_count, if path.is_empty() { "root" } else { path });
        
        Ok(DropboxListResponse {
            entries,
            total_count,
        })
    } else {
        let error_text = response.text().await?;
        Err(format!("Dropbox API error: {}", error_text).into())
    }
}
