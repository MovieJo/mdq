use std::path::PathBuf;

use clap::Parser;

use crate::{Cli, Commands, FindFormat, GetFormat, TreeFormat};

fn parse_ok(args: &[&str]) -> Cli {
    Cli::try_parse_from(args)
        .expect("args should parse")
        .validated()
        .expect("args should validate")
}

#[test]
fn tree_command_uses_documented_defaults() {
    let cli = parse_ok(&["mdq", "tree", "README.md"]);

    match cli.command {
        Commands::Tree(args) => {
            assert_eq!(args.file, PathBuf::from("README.md"));
            assert_eq!(args.format, TreeFormat::AnnotatedMd);
            assert_eq!(args.max_depth, None);
            assert!(!args.no_summary);
        }
        _ => panic!("expected tree command"),
    }
}

#[test]
fn get_command_accepts_documented_flags() {
    let cli = parse_ok(&[
        "mdq",
        "get",
        "README.md",
        "--id",
        "s1-2",
        "--max-lines",
        "10",
        "--with-line-numbers",
    ]);

    match cli.command {
        Commands::Get(args) => {
            assert_eq!(args.file, PathBuf::from("README.md"));
            assert_eq!(args.id, "s1-2");
            assert_eq!(args.format, GetFormat::Text);
            assert_eq!(args.max_lines, Some(10));
            assert!(args.with_line_numbers);
        }
        _ => panic!("expected get command"),
    }
}

#[test]
fn get_command_rejects_line_numbers_for_json() {
    let cli = Cli::try_parse_from([
        "mdq",
        "get",
        "README.md",
        "--id",
        "s1",
        "--format",
        "json",
        "--with-line-numbers",
    ])
    .expect("args should parse");

    let err = cli.validated().expect_err("validation should fail");
    assert_eq!(
        err.message(),
        "--with-line-numbers can only be used with --format text"
    );
}

#[test]
fn numeric_limits_reject_zero() {
    let cli = Cli::try_parse_from(["mdq", "tree", "README.md", "--max-depth", "0"])
        .expect("args should parse");
    let err = cli.validated().expect_err("validation should fail");
    assert_eq!(err.message(), "--max-depth must be greater than 0");

    let cli = Cli::try_parse_from(["mdq", "get", "README.md", "--id", "s1", "--max-lines", "0"])
        .expect("args should parse");
    let err = cli.validated().expect_err("validation should fail");
    assert_eq!(err.message(), "--max-lines must be greater than 0");

    let cli = Cli::try_parse_from(["mdq", "find", "README.md", "install", "--max-matches", "0"])
        .expect("args should parse");
    let err = cli.validated().expect_err("validation should fail");
    assert_eq!(err.message(), "--max-matches must be greater than 0");
}

#[test]
fn find_command_accepts_search_flags() {
    let cli = parse_ok(&[
        "mdq",
        "find",
        "README.md",
        "install",
        "--format",
        "json",
        "--regex",
        "--case-sensitive",
        "--max-matches",
        "5",
    ]);

    match cli.command {
        Commands::Find(args) => {
            assert_eq!(args.file, PathBuf::from("README.md"));
            assert_eq!(args.query, "install");
            assert_eq!(args.format, FindFormat::Json);
            assert!(args.regex);
            assert!(args.case_sensitive);
            assert_eq!(args.max_matches, 5);
        }
        _ => panic!("expected find command"),
    }
}

#[test]
fn find_command_uses_documented_default_max_matches() {
    let cli = parse_ok(&["mdq", "find", "README.md", "install"]);

    match cli.command {
        Commands::Find(args) => {
            assert_eq!(args.file, PathBuf::from("README.md"));
            assert_eq!(args.query, "install");
            assert_eq!(args.format, FindFormat::Text);
            assert!(!args.regex);
            assert!(!args.case_sensitive);
            assert_eq!(args.max_matches, 200);
        }
        _ => panic!("expected find command"),
    }
}

#[test]
fn missing_get_id_is_a_parse_error() {
    let result = Cli::try_parse_from(["mdq", "get", "README.md"]);
    assert!(result.is_err());
}
