use clap::Parser;
use std::path::PathBuf;
use helpman::generate_manpage;

/// CLI tool to generate manpages from a binary's help output
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the binary for which the manpage will be generated
    binary_path: PathBuf,

    /// Name of the binary (used in the manpage header). Defaults to the binary file name.
    #[arg(short = 'n', long)]
    binary_name: Option<String>,

    /// Directory where the generated manpage will be saved (defaults to current working directory)
    #[arg(short = 'o', long, default_value = ".")]
    output_dir: PathBuf,
}

fn main() {
    let args = Args::parse();

    // Use the provided binary name, or default to the file name from the binary path
    let binary_name = args.binary_name.unwrap_or_else(|| {
        args.binary_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown")
            .to_string()
    });

    if let Err(e) = generate_manpage(&args.binary_path, &binary_name, &args.output_dir) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
