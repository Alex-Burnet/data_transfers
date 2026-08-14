use data_transfers::sftp::SftpClient;
use std::{env, fs};

#[test]
fn sftp_upload_and_download() {
    let (host, username, password) = match (
        env::var("SFTP_TEST_HOST").ok(),
        env::var("SFTP_TEST_USERNAME").ok(),
        env::var("SFTP_TEST_PASSWORD").ok(),
    ) {
        (Some(host), Some(username), Some(password)) => {
            (host, username, password)
        }
        _ => {
            println!("Skipping SFTP test: SFTP test environment variables not set");
            return;
        }
    };

    let port = env::var("SFTP_TEST_PORT")
        .unwrap_or_else(|_| "22".to_string());

    let local_upload = "test_sftp_upload.txt";
    let remote_file = "test_sftp_upload.txt";
    let local_download = "test_sftp_download.txt";

    // Create a test file locally
    fs::write(local_upload, b"SFTP integration test")
        .expect("Failed to create local test file");

    // Connect to the SFTP server
    let client = SftpClient::connect(
        &host,
        &port,
        &username,
        &password,
    )
    .expect("Failed to connect to SFTP server");

    // Upload the file
    client
        .local_to_remote(local_upload, remote_file)
        .expect("Failed to upload file");

    // Download it again
    client
        .remote_to_local(remote_file, local_download)
        .expect("Failed to download file");

    // Check that the downloaded file contains what we originally uploaded
    let downloaded = fs::read(local_download)
        .expect("Failed to read downloaded file");

    assert_eq!(downloaded, b"SFTP integration test");

    // Clean up local test files
    let _ = fs::remove_file(local_upload);
    let _ = fs::remove_file(local_download);
}