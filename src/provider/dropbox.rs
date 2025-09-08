use serde_json::json;
use std::path::Path;
use crate::command::data::DropboxUploadResponse;

pub async fn upload_file_to_dropbox(
    access_token: &str,
    path_to_file: &str,
) -> Result<DropboxUploadResponse, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let file_path = Path::new(path_to_file);
    
    // Extract filename from path
    let filename = file_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("Invalid file path")?;
    
    // Read file content
    let file_content = match std::fs::read(path_to_file) {
        Ok(content) => content,
        Err(e) => return Err(format!("Failed to read file: {}", e).into()),
    };
    
    // Create the destination path in Dropbox (in root directory)
    let dropbox_path = format!("/{}", filename);
    
    // Create the API arguments for Dropbox
    let api_args = json!({
        "path": dropbox_path,
        "mode": "add",
        "autorename": true,
        "mute": false,
        "strict_conflict": false
    });
    
    // Send the request
    let response = client
        .post("https://content.dropboxapi.com/2/files/upload")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Dropbox-API-Arg", api_args.to_string())
        .header("Content-Type", "application/octet-stream")
        .body(file_content)
        .send()
        .await?;
    
    if response.status().is_success() {
        let upload_response: DropboxUploadResponse = response.json().await?;
        println!("File '{}' uploaded successfully to: {}", filename, upload_response.path_display);
        Ok(upload_response)
    } else {
        let error_text = response.text().await?;
        Err(format!("Dropbox API error: {}", error_text).into())
    }
}
