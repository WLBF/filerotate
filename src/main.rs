use std::fs::File;
use tracing::info;
use clap::Parser;
use std::io::BufReader;

mod file;
mod rotate;
mod path_rule;
mod regex;

/// A file rotate tool
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// path of job list file
    #[clap(short, long)]
    path: String,

    /// format of job list file
    #[clap(short, long)]
    format: String,
}

fn main() {
    // install global collector configured based on RUST_LOG env var.
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let file = File::open(args.path).expect("invalid config file path");
    let reader = BufReader::new(file);
    let list: Vec<rotate::Rotate> = serde_json::from_reader(reader).expect("json was not well-formatted");

    for ro in list.iter() {
        ro.rotate().unwrap();
    }
}
