use suppaftp::FtpStream;
use std::fs::File;
use std::io::{self, Write};
use anyhow::{ Context, Result };

pub struct FtpClient {
    conn: FtpStream,
}

impl FtpClient {
    /// Opens and authenticates an FTP connection using the configured server credentials.
    /// Returns a result containing an FTPStream
    pub fn connect(host: &str, port: &str, username: &str, password: &str) -> Result<Self> {
        // Set up ftp connection
        let mut conn = FtpStream::connect(format!("{host}:{port}"))
            .with_context(|| format!("Failed to connect to FTP server {host}:{port}"))?;

        // Login to FTP server
        conn.login(username, password)
            .with_context(|| "Failed to authenticate with FTP server")?;

        Ok(Self{ conn })
    }

    /// Downloads a remote file to the local filesystem.
    pub fn remote_to_local(
        &mut self,
        remote_path: &str,
        local_path: &str,
    ) -> Result<()> {
        let mut remote_file = self
            .conn
            .retr_as_buffer(remote_path)
            .with_context(|| {format!("Failed to download remote file {remote_path}")})?;

        let mut local_file = File::create(local_path)
            .with_context(|| {format!("Failed to create local file {local_path}")})?;

        io::copy(&mut remote_file, &mut local_file)
            .with_context(|| {format!("Failed to write local file {local_path}")})?;

        Ok(())
    }

    /// Uploads a local file to remote
    pub fn local_to_remote(
        &mut self,
        local_path: &str,
        remote_path: &str,
    ) -> Result<()> {
        let mut local_file = File::open(local_path)
            .with_context(|| {format!("Failed to open local file {local_path}")})?;

        self.conn
            .put_file(remote_path, &mut local_file)
            .with_context(|| {format!("Failed to upload file to {remote_path}")})?;

        Ok(())
}
}