mod common;

use common::Fixture;
use insta_cmd::assert_cmd_snapshot;

const QUERY: &str = "MATCH (c: Candidate) RETURN c.id LIMIT 5";

//-------------//
//  SUCCESSES  //
//-------------//

#[test]
fn shows_help() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd(["query", "--help"]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Execute a one-off query

    Usage: grafq query [OPTIONS] <QUERY>

    Arguments:
      <QUERY>  Cypher query to execute

    Options:
      -p, --page-results                    Display results via a pager ("less", by default, can be overridden by $GRAFQ_PAGER)
      -b, --bench                           Whether to benchmark the query
          --debug                           Output debug information without doing anything
      -n, --bench-num-runs <NUMBER>         Number of benchmark runs [default: 5]
      -W, --bench-num-warmup-runs <NUMBER>  Number of benchmark warmup runs [default: 3]
      -P, --print-query                     Print query
      -w, --write-results                   Write results to filesystem
      -d, --results-dir <DIRECTORY>         Directory to write results in [default: .grafq]
      -f, --results-format <FORMAT>         Format to write results in [default: json] [possible values: csv, json]
      -h, --help                            Print help

    ----- stderr -----
    "#);
}

#[test]
fn debug_flag_works_for_defaults() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd(["query", "--debug", "-"]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    DEBUG INFO

    command:                    query
    display results via pager:  false
    benchmark:                  false
    print query:                false
    write results:              false

    query:                      -

    ----- stderr -----
    ");
}

#[test]
fn debug_flag_works_with_overridden_flags() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd([
        "query",
        "--bench",
        "--bench-num-runs",
        "10",
        "--bench-num-warmup-runs",
        "5",
        "--print-query",
        "--debug",
        QUERY,
    ]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    DEBUG INFO

    command:                    query
    display results via pager:  false
    benchmark:                  true
    benchmark num runs:         10
    benchmark num warmup runs:  5
    print query:                true
    write results:              false

    query:
    ---
    MATCH (c: Candidate) RETURN c.id LIMIT 5
    ---

    ----- stderr -----
    ");
}

#[test]
fn debug_flag_works_for_write_results_flags() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd([
        "query",
        "--write-results",
        "--results-dir",
        "path/to/results/dir",
        "--results-format",
        "json",
        "--debug",
        QUERY,
    ]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    DEBUG INFO

    command:                    query
    display results via pager:  false
    benchmark:                  false
    print query:                false
    write results:              true
    results directory:          path/to/results/dir
    results format:             json

    query:
    ---
    MATCH (c: Candidate) RETURN c.id LIMIT 5
    ---

    ----- stderr -----
    ");
}

//-------------//
//  FAILURES   //
//-------------//

#[test]
fn fails_if_provided_with_no_db_uri() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd(["query", QUERY]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r#"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    Error: couldn't build db client

    Caused by:
        DB_URI is not set

    grafq requires the environment variable DB_URI to be set.

    - For an AWS Neptune database, use the https scheme. Neptune uses IAM
        authentication, so ensure your AWS credentials are configured correctly (via
        environment variables or the AWS shared config file):

        DB_URI="https://abc.xyz.us-east-1.neptune.amazonaws.com:8182"

    - For a Neo4j database, use the bolt scheme and provide authentication details:

        DB_URI="bolt://127.0.0.1:7687"
        NEO4J_USER="neo4j"
        NEO4J_PASSWORD="your-password"
        NEO4J_DB="neo4j"
    "#);
}

#[test]
fn fails_if_provided_with_invalid_db_uri_scheme() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd(["query", QUERY]);
    cmd.env("DB_URI", "invalid://abc.xyz:8182");

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r#"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    Error: couldn't build db client

    Caused by:
        DB_URI has unsupported scheme: "invalid"

    Only 'bolt' and 'https' schemes are supported by grafq.
    Use bolt for neo4j, and https for AWS Neptune.
    "#);
}

#[test]
fn fails_if_not_provided_with_neo4j_auth() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd(["query", QUERY]);
    cmd.env("DB_URI", "bolt://127.0.0.1:7687");

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r#"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    Error: couldn't build db client

    Caused by:
        environment variable "NEO4J_USER" is missing

    The environment variables NEO4J_USER, NEO4J_PASSWORD, and NEO4J_DB need to be set when connecting
    to a neo4j database (which was determined by the bolt scheme in DB_URI).
    "#);
}

#[test]
fn fails_if_provided_with_incorrect_benchmark_num_runs() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd(["query", "--bench", "--bench-num-runs", "0", QUERY]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: invalid value '0' for '--bench-num-runs <NUMBER>': needs to be greater than 0

    For more information, try '--help'.
    ");
}

#[test]
fn fails_if_both_benchmark_and_write_flags_are_provided() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd(["query", "--bench", "--write-results", QUERY]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    Error: cannot benchmark and write results at the same time
    ");
}

#[test]
fn fails_if_incorrect_results_format_provided() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd([
        "query",
        "--write-results",
        "--results-format",
        "unknown",
        QUERY,
    ]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: invalid value 'unknown' for '--results-format <FORMAT>'
      [possible values: csv, json]

    For more information, try '--help'.
    ");
}
