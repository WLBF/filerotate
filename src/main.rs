use std::fs::File;
use tracing::info;
use clap::Parser;
use std::io::BufReader;
use nix::libc::printf;

mod file;
mod rotate;
mod path_rule;
mod regex;

/// A file rotate tool
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// path of config file
    #[clap(short, long)]
    path: String,
}


fn main() {
    let args = Args::parse();

    let file = File::open(args.path).expect("invalid config file path");
    let reader = BufReader::new(file);
    let arg: rotate::RotateArgs = serde_json::from_reader(reader).expect("json was not well-formatted");
    println!("{:?}", arg);

    // install global collector configured based on RUST_LOG env var.
    tracing_subscriber::fmt::init();

    let number_of_yaks: i32 = 3;
    // this creates a new event, outside of any spans.
    info!(number_of_yaks, "preparing to shave yaks");

    info!(all_yaks_shaved = true, "yak shaving completed.");
}
