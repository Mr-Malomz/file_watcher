use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "file_uploader",
    about = "A CLI tool to upload files to various cloud storage providers.",
    version = "1.0",
    author = "Demola Malomo"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    AWS {
        #[arg(short = 'r', long = "region")]
        region: String,
        #[arg(short = 'b', long = "bucket_name")]
        bucket_name: String,
        #[arg(short = 'p', long = "path_to_file")]
        path_to_file: String,
        #[arg(short = 'k', long = "key")]
        key: String,
    },
    Dropbox {
        #[arg(short = 'a', long = "access_token")]
        access_token: String,
        #[arg(short = 'p', long = "path_to_file")]
        path_to_file: String,
        // #[arg(short = 'k', long = "key")]
        // key: String,
    },
    GoogleDrive {
        #[arg(short = 'a', long = "access_token")]
        access_token: String,
        #[arg(short = 'p', long = "path_to_file")]
        path_to_file: String,
    },
}

#[derive(Debug, Clone)]
pub enum Provider {
    AWS,
    Dropbox,
    GoogleDrive,
}

#[derive(Debug, Clone)]
pub enum Operation {
    Upload,
    ListFiles,
}

#[derive(Debug, Default)]
pub struct AwsConfig {
    pub region: String,
    pub bucket_name: String,
    pub path_to_file: String,
    pub key: String,
}

#[derive(Debug, Default)]
pub struct DropboxConfig {
    pub access_token: String,
    pub path_to_file: String,
    pub key: String,
}

#[derive(Debug, Default)]
pub struct GoogleDriveConfig {
    pub access_token: String,
    pub path_to_file: String,
}

// Response structures for upload operations
#[derive(Debug, serde::Deserialize)]
pub struct AwsUploadResponse {
    // AWS S3 PutObjectOutput doesn't have a standard JSON response structure
    // We'll use a custom structure for consistency
    pub success: bool,
    pub message: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct GoogleDriveUploadResponse {
    pub id: String,
    pub name: String,
    pub web_view_link: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct DropboxUploadResponse {
    pub name: String,
    pub path_lower: String,
    pub path_display: String,
    pub id: String,
    pub client_modified: String,
    pub server_modified: String,
    pub rev: String,
    pub size: u64,
    pub is_downloadable: Option<bool>,
}

// Response structures for list operations
#[derive(Debug, serde::Deserialize)]
pub struct AwsListResponse {
    pub objects: Vec<AwsObject>,
    pub total_count: usize,
}

#[derive(Debug, serde::Deserialize)]
pub struct AwsObject {
    pub key: String,
    pub size: u64,
    pub last_modified: String,
    pub storage_class: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct GoogleDriveListResponse {
    pub files: Vec<GoogleDriveFile>,
    pub total_count: usize,
}

#[derive(Debug, serde::Deserialize)]
pub struct GoogleDriveFile {
    pub id: String,
    pub name: String,
    pub size: Option<String>,
    pub modified_time: Option<String>,
    pub mime_type: Option<String>,
    pub web_view_link: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct DropboxListResponse {
    pub entries: Vec<DropboxEntry>,
    pub total_count: usize,
}

#[derive(Debug, serde::Deserialize)]
pub struct DropboxEntry {
    pub name: String,
    pub path_lower: String,
    pub path_display: String,
    pub id: Option<String>,
    pub size: Option<u64>,
    pub client_modified: Option<String>,
    pub server_modified: Option<String>,
    #[serde(rename = ".tag")]
    pub tag: String, // "file" or "folder"
}
