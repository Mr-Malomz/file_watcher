/// Sends a POST request to the Google Drive API to upload a file using multipart upload.
///
/// # Arguments
///
/// * `client` - An instance of the HTTP client used to make the request.
/// * `access_token` - A string slice containing the OAuth 2.0 access token for authentication.
/// * `file_content` - The content of the file to be uploaded.
///
/// # Returns
///
/// Returns a `Result` containing the HTTP response from the Google Drive API if the request is successful,
/// or an error if the request fails.
///
/// # Errors
///
/// This function will return an error if the HTTP request fails or if the response cannot be parsed.
/// ```

pub async fn upload_file_to_google_drive(
    access_token: &str,
    path_to_file: &str,
) -> Result<(), reqwest::Error> {
    // Create a http client
    let client = reqwest::Client::new();

    // Read the file into a byte vector
    let file_content = std::fs::read(path_to_file).expect("Failed to read file");

    // Create the request
    let response = client
        .post("https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart")
        .header("Authorization ", format!("Bearer {}", access_token))
        .header("Content-Type", "application/json; charset=UTF-8")
        .header("Content-Length", file_content.len())
        .body(file_content)
        .send()
        .await;

    // Check the response
    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                println!("File uploaded successfully");
                Ok(())
            } else {
                Err(resp.text().await.unwrap_err())
            }
        }
        Err(e) => {
            eprintln!("Request failed: {:?}", e);
            Err(e)
        }
    }
}
