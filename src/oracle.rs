//! Oracle database client for exporting query results to CSV/DSV files.
//!
//! This module provides the [`OracleClient`] struct, which wraps an Oracle database
//! connection and provides methods for executing SQL queries and exporting results
//! to local CSV files.
//!
//! # Example
//!
//! ```no_run
//! use data_transfers::oracle::OracleClient;
//!
//! // Connect to Oracle database
//! let client = OracleClient::connect(
//!     "username",
//!     "password",
//!     "localhost:1521/orcl"
//! ).expect("Failed to connect to Oracle");
//!
//! // Export a table to CSV
//! client.table_to_local_file("EMPLOYEES", "employees.csv")
//!     .expect("Failed to export table");
//! ```

use anyhow::{Context, Result};
use csv::{ReaderBuilder, WriterBuilder};
use oracle::Connection;
use std::fs::File;
use std::io::Write;

/// Client for interacting with an Oracle database.
///
/// This struct wraps an Oracle database connection and provides methods for
/// executing SQL queries and exporting results to CSV/DSV files.
///
/// # Thread Safety
///
/// The `OracleClient` is not `Send` or `Sync`. Do not share instances across threads.
///
/// # CSV Format
///
/// - Delimiter: Comma (`,`)
/// - Quoting: Automatic based on field content
/// - NULL values: Represented as empty strings
///
/// # Errors
///
/// This client can fail due to:
/// - Database connection failures
/// - SQL syntax or permission errors
/// - File I/O errors (creation, writing, flushing)
pub struct OracleClient {
    conn: Connection,
}

impl OracleClient {
    /// Opens a connection to an Oracle database.
    ///
    /// # Arguments
    /// * `username` - The Oracle database username
    /// * `password` - The user's password
    /// * `connect_string` - The Oracle connection string in one of these formats:
    ///   - Easy Connect: `"host:port/service_name"` (e.g., `"localhost:1521/orcl"`)
    ///   - TNS descriptor: A full TNS connection descriptor
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The connection string is malformed
    /// - The database is unreachable
    /// - Authentication fails
    /// - Network connectivity issues occur
    ///
    /// # Example
    ///
    /// ```no_run
    /// use data_transfers::oracle::OracleClient;
    ///
    /// let client = OracleClient::connect(
    ///     "scott",
    ///     "tiger",
    ///     "localhost:1521/orcl"
    /// )?;
    /// # Ok::<_, anyhow::Error>(())
    /// ```
    pub fn connect(username: &str, password: &str, connect_string: &str) -> Result<Self> {
        let conn = Connection::connect(
            username,
            password,
            connect_string,
        )?;

        Ok(Self { conn })
    }

    /// Exports all rows from a database view to a local CSV file.
    ///
    /// This is a convenience wrapper around [`query_to_local_file`](OracleClient::query_to_local_file)
    /// that performs a `SELECT *` query on the specified view.
    ///
    /// # Arguments
    /// * `view_name` - The name of the view to export. Case-sensitive depending on database configuration.
    /// * `local_path` - The local file path where the CSV will be written.
    ///   If the file exists, it will be overwritten.
    ///
    /// # CSV Format
    ///
    /// - First row contains column headers
    /// - Delimiter: Comma (`,`)
    /// - NULL values are represented as empty strings
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The view does not exist or is inaccessible
    /// - The query fails to execute
    /// - The local file cannot be created or written
    ///
    /// # Example
    ///
    /// ```no_run
    /// use data_transfers::oracle::OracleClient;
    ///
    /// let client = OracleClient::connect("user", "pass", "localhost:1521/orcl")?;
    /// client.view_to_local_file("ACTIVE_USERS", "active_users.csv")?;
    /// # Ok::<_, anyhow::Error>(())
    /// ```
    pub fn view_to_local_file(
        &self,
        view_name: &str,
        local_path: &str,
    ) -> Result<()> {
        self.query_to_local_file(
            &format!("SELECT * FROM {view_name}"),
            local_path,
        )
    }

    /// Exports all rows from a database table to a local CSV file.
    ///
    /// This is a convenience wrapper around [`query_to_local_file`](OracleClient::query_to_local_file)
    /// that performs a `SELECT *` query on the specified table.
    ///
    /// # Arguments
    /// * `table_name` - The name of the table to export. Case-sensitive depending on database configuration.
    /// * `local_path` - The local file path where the CSV will be written.
    ///   If the file exists, it will be overwritten.
    ///
    /// # CSV Format
    ///
    /// - First row contains column headers
    /// - Delimiter: Comma (`,`)
    /// - NULL values are represented as empty strings
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The table does not exist or is inaccessible
    /// - The query fails to execute
    /// - The local file cannot be created or written
    ///
    /// # Example
    ///
    /// ```no_run
    /// use data_transfers::oracle::OracleClient;
    ///
    /// let client = OracleClient::connect("user", "pass", "localhost:1521/orcl")?;
    /// client.table_to_local_file("EMPLOYEES", "employees.csv")?;
    /// # Ok::<_, anyhow::Error>(())
    /// ```
    pub fn table_to_local_file(
        &self,
        table_name: &str,
        local_path: &str,
    ) -> Result<()> {
        self.query_to_local_file(
            &format!("SELECT * FROM {table_name}"),
            local_path,
        )
    }

    /// Executes a SQL query and writes the results to a local CSV/DSV file.
    ///
    /// # Arguments
    /// * `sql` - The SQL query to execute. Must return a result set (e.g., SELECT statements).
    /// * `local_path` - The local file path where the CSV will be written.
    ///   If the file exists, it will be overwritten.
    ///
    /// # CSV Format
    ///
    /// - First row contains column headers (derived from query result metadata)
    /// - Delimiter: Comma (`,`)
    /// - Quoting: Automatic based on field content
    /// - NULL values: Represented as empty strings
    ///
    /// # Performance Considerations
    ///
    /// - Large result sets may consume significant memory as all rows are buffered
    /// - Consider adding WHERE clauses to limit result set size
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The SQL query is syntactically invalid
    /// - The user lacks permission to execute the query
    /// - The database connection is lost
    /// - The local file cannot be created, written, or flushed
    ///
    /// # Example
    ///
    /// ```no_run
    /// use data_transfers::oracle::OracleClient;
    ///
    /// let client = OracleClient::connect("user", "pass", "localhost:1521/orcl")?;
    ///
    /// // Export filtered results
    /// client.query_to_local_file(
    ///     "SELECT id, name, email FROM users WHERE active = 1",
    ///     "active_users.csv"
    /// )?;
    /// # Ok::<_, anyhow::Error>(())
    /// ```
    pub fn query_to_local_file(
        &self,
        sql: &str,
        local_path: &str,
    ) -> Result<()> {
        let mut stmt = self.conn
            .statement(sql)
            .build()
            .with_context(|| "Failed to prepare Oracle query")?;

        let rows = stmt
            .query(&[])
            .with_context(|| "Failed to execute Oracle query")?;

        // Create CSV writer with comma delimiter
        let mut writer = WriterBuilder::new()
            .delimiter(b',')
            .from_path(local_path)
            .with_context(|| {format!("Failed to create local file {local_path}")})?;

        // Write column headers from query result metadata
        let column_names: Vec<String> = rows
            .column_info()
            .iter()
            .map(|column| column.name().to_string())
            .collect();

        writer
            .write_record(&column_names)
            .with_context(|| "Failed to write CSV headers")?;

        // Iterate through result rows and write each as a CSV record
        for row_result in rows {
            let row = row_result
                .with_context(|| "Failed to read Oracle row")?;

            // Convert SQL values to strings; NULL values become empty strings
            let values: Vec<String> = row
                .sql_values()
                .iter()
                .map(|value| value.to_string())
                .collect();

            writer
                .write_record(&values)
                .with_context(|| "Failed to write CSV record")?;
        }

        // Ensure all data is flushed to disk
        writer
            .flush()
            .with_context(|| "Failed to flush CSV file")?;

        Ok(())
    }



}