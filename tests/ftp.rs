use data_transfers::ftp::FtpClient;
use std::env;
use std::fs;

#[test]
fn ftp_upload_and_download() {
    let (host, username, password) = match (
        env::var("FTP_TEST_HOST").ok(),
        env::var("FTP_TEST_USERNAME").ok(),
        env::var("FTP_TEST_PASSWORD").ok(),
    ) {
        (Some(host), Some(username), Some(password)) => (host, username, password),
        _ => {
            println!("Skipping FTP test: FTP test environment variables not set");
            return;
        }
    };

    let port = env::var("FTP_TEST_PORT").unwrap_or_else(|_| "21".to_string());

    let mut client = FtpClient::connect(
        &host,
        &port,
        &username,
        &password,
    )
    .expect("Failed to connect");

    let local_upload = "test_upload.txt";
    let remote_file = "test_upload.txt";
    let local_download = "test_download.txt";

    fs::write(local_upload, b"FTP integration test")
        .expect("Failed to create test file");

    client
        .local_to_remote(local_upload, remote_file)
        .expect("Failed to upload");

    client
        .remote_to_local(remote_file, local_download)
        .expect("Failed to download");

    let downloaded = fs::read(local_download)
        .expect("Failed to read downloaded file");

    assert_eq!(downloaded, b"FTP integration test");

    // Clean up local files
    let _ = fs::remove_file(local_upload);
    let _ = fs::remove_file(local_download);
}