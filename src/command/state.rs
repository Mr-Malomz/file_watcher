
#[derive(Debug, Default)]
pub struct Provider {
    pub selected_type: String,
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
