// This is free and unencumbered software released into the public domain.

mod config;
mod sysexits;

use crate::sysexits::{exit, Sysexits};
use blobary::{BlobHash, BlobHasher, BlobStore, PersistentBlobStore};
use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use shadow_rs::shadow;
use std::path::{Path, PathBuf};

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
        exit(version().err().unwrap_or_default());
    }

    if options.license {
        exit(license().err().unwrap_or_default());
    }

    if options.verbose || options.debug {
        // TODO: configure tracing
    }

    let result = match &options.command.expect("subcommand is required") {
        Commands::Config {} => Commands::config(),
        Commands::Hash { path } => Commands::hash(path),
        Commands::Init {} => Commands::init(),
        Commands::Check {} => Commands::check(),
        Commands::List {} => Commands::list(),
        Commands::Add { path } => Commands::add(path),
        Commands::Put { text } => Commands::put(text),
        Commands::Get { id } => Commands::get(id),
        Commands::Remove { id } => Commands::remove(id),
        Commands::Pull { url } => Commands::pull(url),
        Commands::Push { url } => Commands::push(url),
        Commands::Sync { url } => Commands::sync(url),
        Commands::Import { path } => Commands::import(path),
        Commands::Export { path } => Commands::export(path),
    };

    exit(result.err().unwrap_or_default());
}

fn _open() -> Result<PersistentBlobStore, Sysexits> {
    match PersistentBlobStore::open_cwd() {
        Ok(store) => Ok(store),
        Err(err) => {
            eprintln!("{}: {}", "blobary", err);
            Err(Sysexits::EX_IOERR)
        }
    }
}

fn version() -> Result<(), Sysexits> {
    let (date, _) = build::BUILD_TIME_3339.split_once('T').unwrap();
    let version_string = format!("{} ({} {})", build::PKG_VERSION, date, build::SHORT_COMMIT,);
    println!("Blobary {}", version_string);
    Ok(())
}

fn license() -> Result<(), Sysexits> {
    let license = include_str!("../../../UNLICENSE");
    println!("{}", license.trim_end());
    Ok(())
}

impl Commands {
    fn config() -> Result<(), Sysexits> {
        Ok(()) // TODO
    }

    fn hash(path: impl AsRef<Path>) -> Result<(), Sysexits> {
        let mut hasher = BlobHasher::new();
        if let Err(_err) = hasher.update_mmap(path) {
            return Err(Sysexits::EX_IOERR);
        }
        let hash = hasher.finalize();
        println!("{}", hash.to_hex());
        Ok(())
    }

    fn init() -> Result<(), Sysexits> {
        Ok(()) // TODO
    }

    fn check() -> Result<(), Sysexits> {
        let _store = _open()?;
        Ok(()) // TODO
    }

    fn list() -> Result<(), Sysexits> {
        let _store = _open()?;
        Ok(()) // TODO
    }

    fn add(path: impl AsRef<Path>) -> Result<(), Sysexits> {
        let mut store = _open()?;
        if let Err(_err) = store.put_file(path) {
            return Err(Sysexits::EX_IOERR);
        }
        Ok(())
    }

    fn put(text: &String) -> Result<(), Sysexits> {
        let mut store = _open()?;
        if let Err(_err) = store.put_string(text) {
            return Err(Sysexits::EX_IOERR);
        }
        Ok(())
    }

    fn get(id: &String) -> Result<(), Sysexits> {
        let store = _open()?;
        let id = BlobHash::from_hex(id).expect("parse hash");
        match store.get_by_hash(id) {
            None => Err(Sysexits::EX_NOINPUT),
            Some(blob) => {
                let mut blob = blob.borrow_mut();
                let mut buffer = String::new();
                if let Err(_err) = blob.read_to_string(&mut buffer) {
                    return Err(Sysexits::EX_IOERR);
                }
                print!("{}", buffer);
                Ok(())
            }
        }
    }

    fn remove(_id: &String) -> Result<(), Sysexits> {
        let _store = _open()?;
        Ok(()) // TODO
    }

    fn pull(_url: &String) -> Result<(), Sysexits> {
        let _store = _open()?;
        Ok(()) // TODO
    }

    fn push(_url: &String) -> Result<(), Sysexits> {
        let _store = _open()?;
        Ok(()) // TODO
    }

    fn sync(_url: &String) -> Result<(), Sysexits> {
        let _store = _open()?;
        Ok(()) // TODO
    }

    fn import(_path: &PathBuf) -> Result<(), Sysexits> {
        let _store = _open()?;
        Ok(()) // TODO
    }

    fn export(_path: &PathBuf) -> Result<(), Sysexits> {
        let _store = _open()?;
        Ok(()) // TODO
    }
}
