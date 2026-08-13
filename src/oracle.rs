use anyhow::{Context, Result};
use csv::{ReaderBuilder, WriterBuilder};
use oracle::Connection;
use std::fs::File;
use std::io::Write;

pub struct OracleClient {
    conn: Connection,
}

impl OracleClient {
    /// Open a connection to the Oracle database.
    ///
    /// # Arguments
    /// * `username` - The oracle username
    /// * `password` - The user password
    /// * `connect_string` - The oracle connection string
    pub fn connect(username: &str, password: &str, connect_string: &str) -> Result<Self> {
        let conn = Connection::connect(
            username,
            password,
            connect_string,
        )?;

        Ok(Self { conn })
    }

    /// Export the contents of a view to a local CSV/DSV file.
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

    /// Export the contents of a table to a local CSV/DSV file.
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

    /// Execute a query and write the results to a local CSV/DSV file.
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

        let mut writer = WriterBuilder::new()
            .delimiter(b',')
            .from_path(local_path)
            .with_context(|| {format!("Failed to create local file {local_path}")})?;

        // Write column names
        let column_names: Vec<String> = rows
            .column_info()
            .iter()
            .map(|column| column.name().to_string())
            .collect();

        writer
            .write_record(&column_names)
            .with_context(|| "Failed to write CSV headers")?;

        for row_result in rows {
            let row = row_result
                .with_context(|| "Failed to read Oracle row")?;

            let values: Vec<String> = row
                .sql_values()
                .iter()
                .map(|value| value.to_string())
                .collect();

            writer
                .write_record(&values)
                .with_context(|| "Failed to write CSV record")?;
        }

        writer
            .flush()
            .with_context(|| "Failed to flush CSV file")?;

        Ok(())
    }



}