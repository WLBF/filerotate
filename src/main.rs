use std::fs::File;
use tracing::{info, debug, error};
use clap::Parser;
use std::io::BufReader;

mod util;
mod rotate;
mod path_rule;
mod regex;
mod byte_size;

#[derive(clap::ArgEnum, Clone, Debug)]
enum Format {
    Json,
    Yaml,
}

/// A file rotate tool
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// path of job list file
    #[clap(short, long)]
    path: String,

    /// format of job list file
    #[clap(arg_enum, short, long, default_value = "yaml")]
    format: Format,
}

fn main() {
    // install global collector configured based on RUST_LOG env var.
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let file = File::open(args.path).expect("invalid config file path");
    let reader = BufReader::new(file);
    let list: Vec<rotate::Rotate> = match args.format {
        Format::Yaml => serde_yaml::from_reader(reader).expect("json was not well-formatted"),
        Format::Json => serde_json::from_reader(reader).expect("json was not well-formatted"),
    };

    for ro in list.iter() {
        info!(path = ro.get_path().to_str().unwrap(), "start to rotate");
        debug!(rotate = format!("{:?}", ro).as_str());
        ro.rotate().map_or_else(
            |e| error!(error = format!("{}", e).as_str(), "failed to rotate"),
            |_| info!("rotate success"),
        );
    }
}
