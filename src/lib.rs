pub mod sftp;
pub mod ftp;
pub mod oracle;

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    #[test]
    fn clients_are_thread_safe() {
        assert_send::<ftp::FtpClient>();
        assert_sync::<ftp::FtpClient>();

        assert_send::<sftp::SftpClient>();
        assert_sync::<sftp::SftpClient>();
    }
}