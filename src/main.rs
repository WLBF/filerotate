use tracing::info;

mod file;
mod rotate;

fn main() {
    // install global collector configured based on RUST_LOG env var.
    tracing_subscriber::fmt::init();

    let number_of_yaks = 3;
    // this creates a new event, outside of any spans.
    info!(number_of_yaks, "preparing to shave yaks");

    info!(all_yaks_shaved = true, "yak shaving completed.");
}
