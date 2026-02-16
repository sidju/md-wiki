use clap::Parser as ClapParser;
use md_wiki::filesystem::RealFileSystem;
use md_wiki::convert_wiki;

/// A minimal static wiki generator using markdown files as input
#[derive(ClapParser, Debug)]
#[command(name = "md-wiki")]
#[command(version, about, long_about = None)]
struct Args {
    /// Directory containing markdown files
    input_directory: String,

    /// Directory where HTML files will be created
    #[arg(default_value = ".")]
    output_directory: String,

    /// Optional path where search index will be written
    #[arg(long = "search-index")]
    search_index: Option<String>,
}

fn main() {
    let args = Args::parse();

    let fs = RealFileSystem;
    match convert_wiki(&fs, &args.input_directory, &args.output_directory, args.search_index.as_deref()) {
        Ok(_) => println!("Successfully converted markdown files to HTML"),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
