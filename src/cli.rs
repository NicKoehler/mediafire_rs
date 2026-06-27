use std::path::PathBuf;

use crate::config::Config;
use clap::{ArgAction, ArgGroup, arg, command, value_parser};

pub fn parse_args() -> Config {
    let matches = command!()
        .author("NicKoehler")
        .color(clap::ColorChoice::Always)
        .arg(
            arg!([URLS] "List of folders or files to download")
                .value_parser(value_parser!(String))
                .num_args(1..),
        )
        .arg(arg!(-i --input <FILE> "File containing URLs").value_parser(value_parser!(PathBuf)))
        .group(
            ArgGroup::new("source")
                .args(["URLS", "input"])
                .required(true),
        )
        .arg(
            arg!(-o --output <OUTPUT> "Output directory")
                .default_value(".")
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            arg!(-m --max <MAX> "Maximum concurrent downloads")
                .default_value("10")
                .value_parser(value_parser!(u64).range(1..=100)),
        )
        .arg(
            arg!(-t --tries <TRIES> "Maximum retries per download")
                .default_value("1")
                .value_parser(value_parser!(u64).range(1..=10)),
        )
        .arg(arg!(-r --reverse "Download largest files first").action(ArgAction::SetTrue))
        .arg(
            arg!(-p --proxy <FILE> "File containing proxy list")
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(arg!(--"proxy-download" "Use proxies for file downloads").action(ArgAction::SetTrue))
        .get_matches();

    Config::new(
        matches
            .get_many("URLS")
            .into_iter()
            .flatten()
            .cloned()
            .collect(),
        matches.get_one::<PathBuf>("input").cloned(),
        matches.get_one::<PathBuf>("output").unwrap().clone(),
        *matches.get_one::<u64>("max").unwrap(),
        *matches.get_one::<u64>("tries").unwrap(),
        *matches.get_one::<bool>("reverse").unwrap(),
        matches.get_one::<PathBuf>("proxy").cloned(),
        *matches.get_one::<bool>("proxy-download").unwrap(),
    )
}
