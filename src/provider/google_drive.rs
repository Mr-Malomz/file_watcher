use serde_json::json;
use std::path::Path;
use crate::command::data::GoogleDriveUploadResponse;


pub async fn upload_file_to_google_drive(
    access_token: &str,
    path_to_file: &str,
) -> Result<GoogleDriveUploadResponse, Box<dyn std::error::Error>> {
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
    
    // Detect MIME type based on file extension
    let mime_type = detect_mime_type(path_to_file);
    
    // Create metadata for the file
    let metadata = json!({
        "name": filename
    });
    
    // Create multipart form
    let form = reqwest::multipart::Form::new()
        .text("metadata", metadata.to_string())
        .part(
            "media",
            reqwest::multipart::Part::bytes(file_content)
                .file_name(filename.to_string())
                .mime_str(&mime_type)?
        );
    
    // Send the request
    let response = client
        .post("https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart")
        .header("Authorization", format!("Bearer {}", access_token))
        .multipart(form)
        .send()
        .await?;
    
    if response.status().is_success() {
        let upload_response: GoogleDriveUploadResponse = response.json().await?;
        println!("File '{}' uploaded successfully with ID: {}", filename, upload_response.id);
        Ok(upload_response)
    } else {
        let error_text = response.text().await?;
        Err(format!("Google Drive API error: {}", error_text).into())
    }
}

/// Detects MIME type based on file extension
fn detect_mime_type(file_path: &str) -> String {
    let path = Path::new(file_path);
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("txt") => "text/plain".to_string(),
        Some("json") => "application/json".to_string(),
        Some("pdf") => "application/pdf".to_string(),
        Some("png") => "image/png".to_string(),
        Some("jpg") | Some("jpeg") => "image/jpeg".to_string(),
        Some("gif") => "image/gif".to_string(),
        Some("mp4") => "video/mp4".to_string(),
        Some("mp3") => "audio/mpeg".to_string(),
        Some("zip") => "application/zip".to_string(),
        Some("html") => "text/html".to_string(),
        Some("css") => "text/css".to_string(),
        Some("js") => "application/javascript".to_string(),
        Some("xml") => "application/xml".to_string(),
        Some("csv") => "text/csv".to_string(),
        _ => "application/octet-stream".to_string(),
    }
}
