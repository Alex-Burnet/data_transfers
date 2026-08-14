use data_transfers::oracle::OracleClient;
use std::{env, fs};

/// Creates an Oracle client using the integration-test environment variables.
///
/// These tests are marked #[ignore], so they only run when explicitly requested:
///
///     cargo test --test oracle -- --ignored
fn create_test_client() -> OracleClient {
    let username = env::var("ORACLE_TEST_USERNAME")
        .expect("ORACLE_TEST_USERNAME not set");

    let password = env::var("ORACLE_TEST_PASSWORD")
        .expect("ORACLE_TEST_PASSWORD not set");

    let connect_string = env::var("ORACLE_TEST_CONNECT_STRING")
        .expect("ORACLE_TEST_CONNECT_STRING not set");

    OracleClient::connect(
        &username,
        &password,
        &connect_string,
    )
    .expect("Failed to connect to Oracle")
}

/// Removes a test output file if it exists.
fn cleanup_file(path: &str) {
    let _ = fs::remove_file(path);
}


#[test]
#[ignore]
fn oracle_query_to_local_file() {
    let client = create_test_client();

    let output = "oracle_test_query.csv";

    cleanup_file(output);

    client
        .query_to_local_file(
            "SELECT 1 AS ID, 'Alice' AS NAME FROM DUAL",
            output,
        )
        .expect("Failed to export query results");

    let contents = fs::read_to_string(output)
        .expect("Failed to read output file");

    assert_eq!(
        contents,
        "ID,NAME\n1,Alice\n"
    );

    cleanup_file(output);
}


// To be added back in when data type handling is added
/* 
#[test]
#[ignore]
fn oracle_query_handles_null_values() {
    let client = create_test_client();

    let output = "oracle_test_null.csv";

    cleanup_file(output);

    client
        .query_to_local_file(
            "SELECT
                1 AS ID,
                'Alice' AS NAME,
                NULL AS DESCRIPTION
             FROM DUAL",
            output,
        )
        .expect("Failed to export query results");

    let contents = fs::read_to_string(output)
        .expect("Failed to read output file");

    assert_eq!(
        contents,
        "ID,NAME,DESCRIPTION\n1,Alice,\n"
    );

    cleanup_file(output);
}
*/

#[test]
#[ignore]
fn oracle_query_handles_csv_quoting() {
    let client = create_test_client();

    let output = "oracle_test_quoting.csv";

    cleanup_file(output);

    client
        .query_to_local_file(
            r#"SELECT
                1 AS ID,
                'Alice, Smith' AS NAME,
                'Hello "world"' AS DESCRIPTION
               FROM DUAL"#,
            output,
        )
        .expect("Failed to export query results");

    let contents = fs::read_to_string(output)
        .expect("Failed to read output file");

    assert_eq!(
        contents,
        "ID,NAME,DESCRIPTION\n\
         1,\"Alice, Smith\",\"Hello \"\"world\"\"\"\n"
    );

    cleanup_file(output);
}


#[test]
#[ignore]
fn oracle_query_handles_multiple_rows() {
    let client = create_test_client();

    let output = "oracle_test_multiple_rows.csv";

    cleanup_file(output);

    client
        .query_to_local_file(
            "SELECT 1 AS ID, 'Alice' AS NAME FROM DUAL
             UNION ALL
             SELECT 2 AS ID, 'Bob' AS NAME FROM DUAL
             UNION ALL
             SELECT 3 AS ID, 'Charlie' AS NAME FROM DUAL
             ORDER BY ID",
            output,
        )
        .expect("Failed to export query results");

    let contents = fs::read_to_string(output)
        .expect("Failed to read output file");

    assert_eq!(
        contents,
        "ID,NAME\n\
         1,Alice\n\
         2,Bob\n\
         3,Charlie\n"
    );

    cleanup_file(output);
}


#[test]
#[ignore]
fn oracle_invalid_query_returns_error() {
    let client = create_test_client();

    let output = "oracle_test_invalid.csv";

    cleanup_file(output);

    let result = client.query_to_local_file(
        "SELECT THIS_COLUMN_DOES_NOT_EXIST FROM DUAL",
        output,
    );

    assert!(
        result.is_err(),
        "Expected invalid Oracle query to return an error"
    );

    cleanup_file(output);
}


#[test]
#[ignore]
fn oracle_query_fails_when_local_path_is_invalid() {
    let client = create_test_client();

    let result = client.query_to_local_file(
        "SELECT 1 AS ID FROM DUAL",
        "/this/path/should/not/exist/oracle_test.csv",
    );

    assert!(
        result.is_err(),
        "Expected invalid local path to return an error"
    );
}