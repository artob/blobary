// This is free and unencumbered software released into the public domain.

mod config;
mod sysexits;

use crate::sysexits::{exit, Sysexits};
use blobary::{BlobHash, BlobStore, PersistentBlobStore};
use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use shadow_rs::shadow;
use std::path::PathBuf;

shadow!(build);

/// Blobary command-line interface (CLI)
#[derive(Parser, Debug)]
#[command(name = "Blobary", about)]
#[command(arg_required_else_help = true)]
struct Options {
    /// Enable debugging output
    #[clap(short = 'd', long, value_parser)]
    debug: bool,

    /// Show license information
    #[clap(long, value_parser)]
    license: bool,

    // Enable verbose output
    #[clap(short = 'v', long, value_parser)]
    verbose: bool,

    /// Print version information
    #[clap(short = 'V', long, value_parser)]
    version: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Show the current configuration
    Config {},
    /// Print out the hash of a file
    #[clap(aliases = &["id", "identify"])]
    Hash { path: PathBuf },
    /// Initialize `$HOME/.config/blobary`
    Init {},
    /// Check the repository integrity
    Check {},
    /// List blobs in the repository
    #[clap(alias = "ls")]
    List {},
    /// Add a file to the repository
    Add { path: PathBuf },
    /// Put text into the repository
    Put { text: String },
    /// Fetch a blob by its hash
    #[clap(alias = "cat")]
    Get { id: String },
    /// Remove a blob by its hash
    #[clap(aliases = &["rm", "del", "delete"])]
    Remove { id: String },
    /// Pull blobs from another repository
    Pull { url: String },
    /// Push blobs to another repository
    Push { url: String },
    /// Sync blobs with another repository
    Sync { url: String },
    /// TODO
    Import { path: PathBuf },
    /// TODO
    Export { path: PathBuf },
}

pub fn main() {
    // Load environment variables from `.env`:
    dotenv().ok();

    // Expand wildcards and @argfiles:
    let args = wild::args_os();
    let args = argfile::expand_args_from(args, argfile::parse_fromfile, argfile::PREFIX).unwrap();

    // Parse command-line options:
    let options = Options::parse_from(args);

    if options.version {
        exit(version());
    }

    if options.license {
        exit(license());
    }

    if options.verbose || options.debug {
        // TODO: configure tracing
    }

    let result = match &options.command.expect("command is required") {
        Commands::Config {} => config(),
        Commands::Hash { path } => hash(path),
        Commands::Init {} => init(),
        Commands::Check {} => check(),
        Commands::List {} => list(),
        Commands::Add { path } => add(path),
        Commands::Put { text } => put(text),
        Commands::Get { id } => get(id),
        Commands::Remove { id } => remove(id),
        Commands::Pull { url } => pull(url),
        Commands::Push { url } => push(url),
        Commands::Sync { url } => sync(url),
        Commands::Import { path } => import(path),
        Commands::Export { path } => export(path),
    };

    exit(result);
}

fn version() -> Sysexits {
    let (date, _) = build::BUILD_TIME_3339.split_once('T').unwrap();
    let version_string = format!("{} ({} {})", build::PKG_VERSION, date, build::SHORT_COMMIT,);
    println!("Blobary {}", version_string);
    Sysexits::EX_OK
}

fn license() -> Sysexits {
    let license = include_str!("../../../UNLICENSE");
    println!("{}", license.trim_end());
    Sysexits::EX_OK
}

fn config() -> Sysexits {
    Sysexits::EX_OK // TODO
}

fn hash(_path: &PathBuf) -> Sysexits {
    Sysexits::EX_OK // TODO
}

fn init() -> Sysexits {
    Sysexits::EX_OK // TODO
}

fn check() -> Sysexits {
    Sysexits::EX_OK // TODO
}

fn list() -> Sysexits {
    Sysexits::EX_OK // TODO
}

fn add(_path: &PathBuf) -> Sysexits {
    Sysexits::EX_OK // TODO
}

fn put(text: &String) -> Sysexits {
    let mut store = PersistentBlobStore::open_cwd().expect("open cwd");
    if let Err(_err) = store.put_string(text) {
        return Sysexits::EX_IOERR;
    }
    Sysexits::EX_OK
}

fn get(id: &String) -> Sysexits {
    let store = PersistentBlobStore::open_cwd().expect("open cwd");
    let id = BlobHash::from_hex(id).expect("parse hash");
    match store.get_by_hash(id) {
        None => Sysexits::EX_NOINPUT,
        Some(blob) => {
            let mut blob = blob.borrow_mut();
            let mut buffer = String::new();
            if let Err(_err) = blob.read_to_string(&mut buffer) {
                return Sysexits::EX_IOERR;
            }
            println!("{}", buffer);
            Sysexits::EX_OK
        }
    }
}

fn remove(_id: &String) -> Sysexits {
    Sysexits::EX_OK // TODO
}

fn pull(_url: &String) -> Sysexits {
    Sysexits::EX_OK // TODO
}

fn push(_url: &String) -> Sysexits {
    Sysexits::EX_OK // TODO
}

fn sync(_url: &String) -> Sysexits {
    Sysexits::EX_OK // TODO
}

fn import(_path: &PathBuf) -> Sysexits {
    Sysexits::EX_OK // TODO
}

fn export(_path: &PathBuf) -> Sysexits {
    Sysexits::EX_OK // TODO
}
