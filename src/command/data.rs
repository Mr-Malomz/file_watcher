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

#[derive(Debug, Default)]
pub struct AWS_Config {
    pub region: String,
    pub bucket_name: String,
    pub path_to_file: String,
    pub key: String,
}

#[derive(Debug, Default)]
pub struct Dropbox_Config {
    pub access_token: String,
    pub path_to_file: String,
    pub key: String,
}

#[derive(Debug, Default)]
pub struct GoogleDrive_Config {
    pub access_token: String,
    pub path_to_file: String,
}
