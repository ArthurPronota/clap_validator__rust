/*
Пример запуска:
cargo run -- -p abc.file -t 100 --config Cargo.toml -e abg@m.com
*/
use clap::{Parser, value_parser} ;
use std::{path::PathBuf, str::FromStr} ;


#[derive(Debug, Parser)]
struct CliArgs {

    /// Path to scan for cleanup
    #[arg(short = 'p', long = "path")]
    path: PathBuf,

    /// Control threshold
    #[arg(
        short = 't', 
        long = "threshold", 
        value_parser = value_parser!(u8)    // макрос создающий парзер для u8
                            .range(0..=100) // сужает поддерживаемый диапазон
      )
    ]
    threshold:  u8,

    /// Config file
    #[arg(short = 'c', long = "config", value_parser = parse_config)]
    config:    PathBuf,

    /// Email
    #[arg(short = 'e', long = "email", value_parser = parse_email)]
    email:     String,
}

// проверка конфигурационного файла
fn parse_config(s: &str) ->Result<PathBuf, String> {
    match PathBuf::from_str(s) {
        Ok(p) => {
            if p.is_file() {
                Ok(p)
            } else {
                Err(format!("Not found file {:?}", p))
            }
        },
        Err(err) => Err(format!("{}", err)),
    }
}

// проверка email
fn parse_email(e: &str) ->Result<String, String> {
    match e.split_once('@') {
        Some((user, domain)) if !user.is_empty() && domain.contains('.')  => Ok(e.to_string()),
        _ => Err(format!("Invalid email: {e}"))
    }
}

fn main() {
    let args = CliArgs::parse() ;

    dbg!(args) ;
}
