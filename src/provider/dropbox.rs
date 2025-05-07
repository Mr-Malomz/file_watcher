/// Sends a POST request to the Dropbox API to upload a file.
///
/// This function uses the Dropbox API endpoint `https://content.dropboxapi.com/2/files/upload`
/// to upload a file to the user's Dropbox account. The request includes the following headers:
/// - `Authorization`: Contains the Bearer token for authentication.
/// - `Dropbox-API-Arg`: Specifies the file path, upload mode, and other options in JSON format.
/// - `Content-Type`: Indicates the content type of the file being uploaded.
///
/// The file content is sent as the body of the request.
///
/// # Steps to Obtain Access Token
/// 1. Navigate to the Dropbox App Console and log in.
/// 2. Click "Create App" and select the desired access type (e.g., "App folder" or "Full Dropbox").
/// 3. After creating the app, you'll be directed to the app's settings page.
/// 4. Scroll down to the OAuth 2 section.
/// 5. Click the "Generate" button next to Generated access token.
///
/// # Parameters
/// - `client`: An HTTP client instance used to send the request.
/// - `access_token`: A string containing the Dropbox API access token for authentication.
/// - `file_content`: The binary content of the file to be uploaded.
///
/// # Returns
/// A `Response` object representing the result of the HTTP request.
///
/// # Errors
/// This function will return an error if the HTTP request fails or if the Dropbox API
/// returns an error response.
/// ```


pub async fn upload_file_to_dropbox(
    access_token: &str,
    path_to_file: &str,
) -> Result<(), reqwest::Error> {
    // Create a http client
    let client = reqwest::Client::new();

    // Read the file into a byte vector
    let file_content = std::fs::read(path_to_file).expect("Failed to read file");

    // Create the request
    let response = client
        .post("https://content.dropboxapi.com/2/files/upload")
        .header("Authorization", format!("Bearer {}", access_token))
        .header(
            "Dropbox-API-Arg",
            r#"{"path":"/file.txt","mode":"add","autorename":true,"mute":false}"#,
        )
        .header("Content-Type", "application/octet-stream")
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
