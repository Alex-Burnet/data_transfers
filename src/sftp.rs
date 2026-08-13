use anyhow::{Context, Result};
use ssh2::Session;
use std::{fs::File, io, net::TcpStream, path::Path};

pub struct SFTPClient {
    conn: SFTP,
}

impl SFTPClient {
    pub fn connect(host: &str, port: &str, username: &str, password: &str) -> Result<Self> {
        let tcp = TcpStream::connect(format!("{host}:{port}"))
            .with_context(|| format!("Failed to connect to SFTP server {host}:{port}"))?;

        let mut session = Session::new()?;
        session.set_tcp_stream(tcp);
        session.handshake()?;

        session
            .userauth_password(username, password)
            .with_context(|| "Failed to authenticate with SFTP server")?;

        if !session.authenticated() {
            anyhow::bail!("Authentication failed");
        }

        let sftp = session.sftp()?;

        Ok(Self{ conn: sftp })
    }

    pub fn local_to_remote(remote_path: &str, local_path: &str) -> Result<> {
        let mut local =
            File::open(local_path).with_context(|| format!("Failed to open {}", local_path))?;

        let mut remote = sftp
            .create(Path::new(remote_path))
            .with_context(|| format!("Failed to create remote file {}", remote_path))?;

        io::copy(&mut local, &mut remote)
            .with_context(|| {format!("Failed to copy {} to {}", local_path, remote_path)})?;

        Ok(())
    }

    pub fn remote_to_local(remote_path: &str, local_path: &str) -> Result<()> {
        let mut remote = sftp
            .open(Path::new(remote_path))
            .with_context(|| format!("Failed to open remote file {}", remote_path))?;

        let mut local = File::create(local_path)
            .with_context(|| format!("Failed to create {}", local_path))?;

        io::copy(&mut remote, &mut local)
            .with_context(|| {format!("Failed to copy {} to {}", remote_path, local_path)})?;

        Ok(())
    }
}