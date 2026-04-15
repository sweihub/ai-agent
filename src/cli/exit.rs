// Source: /data/home/swei/claudecode/openclaudecode/src/cli/exit.ts
//! CLI exit helpers

/// Write an error message to stderr (if given) and exit with code 1.
#[macro_export]
macro_rules! cli_error {
    ($msg:expr) => {{
        eprintln!("{}", $msg);
        std::process::exit(1);
    }};
    () => {{
        std::process::exit(1);
    }};
}

/// Write a message to stdout (if given) and exit with code 0.
#[macro_export]
macro_rules! cli_ok {
    ($msg:expr) => {{
        println!("{}", $msg);
        std::process::exit(0);
    }};
    () => {{
        std::process::exit(0);
    }};
}
