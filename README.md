# data_transfers

A reusable Rust library for transferring data between databases, local filesystems, and remote file-transfer servers.

## Features

- FTP file transfers
  - Connect and authenticate with an FTP server
  - Download remote files
  - Upload local files
- SFTP file transfers
  - Connect and authenticate with an SFTP server
  - Download remote files
  - Upload local files
- Oracle database access
  - Connect to Oracle
  - Export query results to CSV files
  - Export tables to CSV files
  - Export views to CSV files
- Error handling using `anyhow`
- Integration tests for external services

## Installation

Add the crate to your application's `Cargo.toml`:

```toml
[dependencies]
data_transfers = "0.1"
```

## FTP

### Connecting

```rust
use data_transfers::ftp::FtpClient;

let mut client = FtpClient::connect(
    "ftp.example.com",
    "21",
    "username",
    "password",
)?;
```

### Downloading a file

```rust
client.remote_to_local(
    "/remote/file.txt",
    "local/file.txt",
)?;
```

### Uploading a file

```rust
client.local_to_remote(
    "local/file.txt",
    "/remote/file.txt",
)?;
```

### Complete example

```rust
use data_transfers::ftp::FtpClient;

fn main() -> anyhow::Result<()> {
    let mut client = FtpClient::connect(
        "ftp.example.com",
        "21",
        "username",
        "password",
    )?;

    client.remote_to_local(
        "/remote/file.txt",
        "local/file.txt",
    )?;

    client.local_to_remote(
        "local/other_file.txt",
        "/remote/other_file.txt",
    )?;

    Ok(())
}
```

The FTP connection remains open after `connect()`, allowing multiple operations to be performed using the same connection.

## SFTP

SFTP uses SSH and normally operates on port `22`.

### Connecting

```rust
use data_transfers::sftp::SftpClient;

let client = SftpClient::connect(
    "sftp.example.com",
    "22",
    "username",
    "password",
)?;
```

### Downloading a file

```rust
client.remote_to_local(
    "/remote/file.txt",
    "local/file.txt",
)?;
```

### Uploading a file

```rust
client.local_to_remote(
    "local/file.txt",
    "/remote/file.txt",
)?;
```

### Complete example

```rust
use data_transfers::sftp::SftpClient;

fn main() -> anyhow::Result<()> {
    let client = SftpClient::connect(
        "sftp.example.com",
        "22",
        "username",
        "password",
    )?;

    client.local_to_remote(
        "local/file.txt",
        "/remote/file.txt",
    )?;

    client.remote_to_local(
        "/remote/other_file.txt",
        "local/other_file.txt",
    )?;

    Ok(())
}
```

## Oracle

The Oracle client provides functionality for connecting to Oracle databases and exporting query results to CSV files.

### Connecting

```rust
use data_transfers::oracle::OracleClient;

let client = OracleClient::connect(
    "username",
    "password",
    "localhost:1521/orcl",
)?;
```

The connection string can use an Oracle Easy Connect string:

```text
host:port/service_name
```

For example:

```text
oracle.example.com:1521/ORCL
```

A full Oracle TNS connection descriptor can also be supplied.

### Exporting a query to CSV

```rust
client.query_to_local_file(
    "SELECT id, name FROM employees",
    "employees.csv",
)?;
```

The resulting CSV contains the column names as the first row:

```csv
ID,NAME
1,Alice
2,Bob
```

### Exporting a table

```rust
client.table_to_local_file(
    "EMPLOYEES",
    "employees.csv",
)?;
```

This is equivalent to:

```sql
SELECT * FROM EMPLOYEES
```

### Exporting a view

```rust
client.view_to_local_file(
    "ACTIVE_EMPLOYEES",
    "active_employees.csv",
)?;
```

This is equivalent to:

```sql
SELECT * FROM ACTIVE_EMPLOYEES
```

### CSV behaviour

Oracle query results are written using the `csv` crate.

- Delimiter: `,`
- Column headers are included
- Values containing commas are quoted
- Values containing quotes are escaped according to CSV rules
- Oracle `NULL` values are written as empty fields
- Existing files are overwritten

For example:

```csv
ID,NAME,DESCRIPTION
1,Alice,
2,"Smith, Bob","Example description"
```

## Error handling

All public operations return:

```rust
anyhow::Result<T>
```

This allows applications using the crate to propagate errors using the `?` operator:

```rust
use data_transfers::ftp::FtpClient;

fn transfer_file() -> anyhow::Result<()> {
    let mut ftp = FtpClient::connect(
        "ftp.example.com",
        "21",
        "username",
        "password",
    )?;

    ftp.remote_to_local(
        "/remote/file.csv",
        "file.csv",
    )?;

    Ok(())
}
```

Errors include contextual information to help identify the operation that failed.

For example:

```text
Failed to download remote file /remote/file.csv
```

## Testing

The crate contains unit tests and integration tests.

### Standard tests

Run the standard test suite with:

```bash
cargo test
```

Tests that require external services are marked with `#[ignore]`.

This means they are not run by a normal `cargo test` and will appear as:

```text
test oracle_query_to_local_file ... ignored
```

This prevents the standard test suite from requiring access to an FTP, SFTP, or Oracle server.

### Oracle integration tests

Oracle integration tests require access to an Oracle database.

Set the following environment variables:

```bash
export ORACLE_TEST_USERNAME='username'
export ORACLE_TEST_PASSWORD='password'
export ORACLE_TEST_CONNECT_STRING='host:1521/service'
```

Then run the Oracle integration tests:

```bash
cargo test --test oracle -- --ignored
```

The Oracle integration tests use Oracle's built-in `DUAL` table and do not require any additional database objects to be created.

### FTP integration tests

Set:

```bash
export FTP_TEST_HOST='ftp.example.com'
export FTP_TEST_PORT='21'
export FTP_TEST_USERNAME='username'
export FTP_TEST_PASSWORD='password'
```

Then run:

```bash
cargo test --test ftp -- --ignored
```

### SFTP integration tests

Set:

```bash
export SFTP_TEST_HOST='sftp.example.com'
export SFTP_TEST_PORT='22'
export SFTP_TEST_USERNAME='username'
export SFTP_TEST_PASSWORD='password'
```

Then run:

```bash
cargo test --test sftp -- --ignored
```

Integration tests should be run against dedicated test systems where possible.

Do not commit passwords or other credentials to the repository.

## Project structure

```text
data_transfers/
├── src/
│   ├── lib.rs
│   ├── ftp.rs
│   ├── sftp.rs
│   └── oracle.rs
│
├── tests/
│   ├── ftp.rs
│   ├── sftp.rs
│   └── oracle.rs
│
├── Cargo.toml
└── README.md
```

## Design

The crate provides lightweight clients for common data-transfer operations.

Applications can compose these clients to implement larger data-transfer workflows.

For example:

```text
Oracle
   |
   | Query
   v
Rust application
   |
   | Create/process files
   v
Local filesystem
   |
   | Upload
   v
FTP / SFTP
```

The crate is responsible for the underlying connection and transfer operations, while the consuming application remains responsible for its business logic and workflow.

## Security

Credentials should be supplied by the consuming application rather than hard-coded into source code.

For example:

```rust
let username = std::env::var("FTP_USERNAME")?;
let password = std::env::var("FTP_PASSWORD")?;
```

Applications should use an appropriate secret-management mechanism for production deployments.

## Dependencies

The crate currently uses:

- `anyhow` - error handling
- `suppaftp` - FTP
- `ssh2` - SFTP/SSH
- `oracle` - Oracle database access
- `csv` - CSV generation

## License

Add your chosen license here.

For example:

```text
MIT License
```