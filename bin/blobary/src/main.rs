// This is free and unencumbered software released into the public domain.

mod config;
mod sysexits;

use crate::sysexits::{exit, Sysexits};
use blobary::{
    BlobHash, BlobHasher, BlobIterator, BlobStore, BlobStoreExt, PersistentBlobStore,
    DEFAULT_MIME_TYPE,
};
use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use shadow_rs::shadow;
use std::{
    fs::File,
    io::{stdout, Write},
    ops::DerefMut,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use tar::{EntryType, Header};

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
    Hash { paths: Vec<PathBuf> },
    /// Initialize `$HOME/.config/blobary`
    Init {},
    /// Check the repository integrity
    Check {},
    /// Compact and compress the repository
    Compact {},
    /// List blobs in the repository
    #[clap(alias = "ls")]
    List {},
    /// Add a file to the repository
    Add { paths: Vec<PathBuf> },
    /// Put text into the repository
    Put { text: String },
    /// Fetch a blob by its hash
    #[clap(alias = "cat")]
    Get { ids: Vec<String> },
    /// Remove a blob by its hash
    #[clap(aliases = &["rm", "del", "delete"])]
    Remove { ids: Vec<String> },
    /// Pull blobs from another repository
    Pull { url: String },
    /// Push blobs to another repository
    Push { url: String },
    /// Sync blobs with another repository
    Sync { url: String },
    /// TODO
    Import { paths: Vec<PathBuf> },
    /// TODO
    Export { path: Option<PathBuf> },
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

    let subcommand = &options.command;
    let result = match subcommand.as_ref().expect("subcommand is required") {
        Commands::Config {} => Commands::config(&options),
        Commands::Hash { paths } => Commands::hash(paths, &options),
        Commands::Init {} => Commands::init(&options),
        Commands::Check {} => Commands::check(&options),
        Commands::Compact {} => Commands::compact(&options),
        Commands::List {} => Commands::list(&options),
        Commands::Add { paths } => Commands::add(paths, &options),
        Commands::Put { text } => Commands::put(text, &options),
        Commands::Get { ids } => Commands::get(ids, &options),
        Commands::Remove { ids } => Commands::remove(ids, &options),
        Commands::Pull { url } => Commands::pull(url, &options),
        Commands::Push { url } => Commands::push(url, &options),
        Commands::Sync { url } => Commands::sync(url, &options),
        Commands::Import { paths } => Commands::import(paths, &options),
        Commands::Export { path } => Commands::export(path, &options),
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
    fn config(_options: &Options) -> Result<(), Sysexits> {
        Ok(()) // TODO
    }

    fn hash(paths: &Vec<impl AsRef<Path>>, _options: &Options) -> Result<(), Sysexits> {
        if paths.is_empty() {
            return Err(Sysexits::EX_USAGE); // TODO: stdin
        } else {
            for path in paths {
                let mut hasher = BlobHasher::new();
                if let Err(_err) = hasher.update_mmap(path) {
                    return Err(Sysexits::EX_IOERR);
                }
                let hash = hasher.finalize();
                println!("{}", hash.to_hex());
            }
        }
        Ok(())
    }

    fn init(_options: &Options) -> Result<(), Sysexits> {
        Ok(()) // TODO
    }

    fn check(_options: &Options) -> Result<(), Sysexits> {
        let _store = _open()?;
        Ok(()) // TODO
    }

    fn compact(_options: &Options) -> Result<(), Sysexits> {
        let _store = _open()?;
        Ok(()) // TODO
    }

    fn list(options: &Options) -> Result<(), Sysexits> {
        let mut store = _open()?;
        for blob in BlobIterator::new(&mut store) {
            let mut blob = blob.borrow_mut();
            let blob_hash = blob.hash()?;
            let blob_hash = blob_hash.to_hex();
            if options.verbose || options.debug {
                let blob_size = blob.size()?;
                let blob_type = blob.mime_type()?.unwrap_or(DEFAULT_MIME_TYPE);
                println!("{}\t{}\t{}", blob_hash, blob_size, blob_type);
            } else {
                println!("{}", blob_hash);
            }
        }
        Ok(())
    }

    fn add(paths: &Vec<impl AsRef<Path>>, _options: &Options) -> Result<(), Sysexits> {
        if paths.is_empty() {
            return Err(Sysexits::EX_USAGE); // TODO: stdin
        } else {
            for path in paths {
                let mut store = _open()?;
                let result = store.put_file(path);
                if let Err(_err) = result {
                    return Err(Sysexits::EX_IOERR);
                }
                let blob_id = result.unwrap();
                let blob_hash = store.id_to_hash(blob_id).unwrap();
                println!("{}", blob_hash.to_hex());
            }
        }
        Ok(())
    }

    fn put(text: &String, _options: &Options) -> Result<(), Sysexits> {
        let mut store = _open()?;
        let result = store.put_string(text);
        if let Err(_err) = result {
            return Err(Sysexits::EX_IOERR);
        }
        let blob_id = result.unwrap();
        let blob_hash = store.id_to_hash(blob_id).unwrap();
        println!("{}", blob_hash.to_hex());
        Ok(())
    }

    fn get(ids: &Vec<String>, _options: &Options) -> Result<(), Sysexits> {
        let store = _open()?;
        for id in ids {
            let id = BlobHash::from_hex(id).expect("parse hash");
            match store.get_by_hash(id) {
                None => return Err(Sysexits::EX_NOINPUT),
                Some(blob) => {
                    let mut blob = blob.borrow_mut();
                    let mut stdout = stdout().lock();
                    std::io::copy(blob.deref_mut(), &mut stdout).unwrap();
                }
            }
        }
        Ok(())
    }

    fn remove(ids: &Vec<String>, _options: &Options) -> Result<(), Sysexits> {
        let _store = _open()?;
        for _id in ids {
            // TODO
        }
        Ok(())
    }

    fn pull(_url: &String, _options: &Options) -> Result<(), Sysexits> {
        let _store = _open()?;
        Ok(()) // TODO
    }

    fn push(_url: &String, _options: &Options) -> Result<(), Sysexits> {
        let _store = _open()?;
        Ok(()) // TODO
    }

    fn sync(_url: &String, _options: &Options) -> Result<(), Sysexits> {
        let _store = _open()?;
        Ok(()) // TODO
    }

    fn import(paths: &Vec<impl AsRef<Path>>, _options: &Options) -> Result<(), Sysexits> {
        let _store = _open()?;
        if paths.is_empty() {
            return Err(Sysexits::EX_USAGE); // TODO: stdin
        } else {
            for _path in paths {
                // TODO
            }
        }
        Ok(())
    }

    fn export(path: &Option<impl AsRef<Path>>, _options: &Options) -> Result<(), Sysexits> {
        let mut store = _open()?;
        let file: Box<dyn Write> = if path.is_none() {
            Box::new(stdout())
        } else {
            Box::new(File::create(path.as_ref().unwrap()).unwrap())
        };
        let blob_mtime: u64 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .try_into()
            .unwrap();
        let mut tarball = tar::Builder::new(file);
        for blob in BlobIterator::new(&mut store) {
            let mut blob = blob.borrow_mut();
            let blob_size = blob.size()?;
            let blob_hash = blob.hash()?;
            let mut file_head = Header::new_ustar();
            file_head.set_entry_type(EntryType::Regular);
            file_head.set_path(blob_hash.to_hex().as_str())?;
            file_head.set_size(blob_size);
            file_head.set_mtime(blob_mtime);
            file_head.set_mode(0o444);
            file_head.set_username("root")?;
            file_head.set_groupname("root")?;
            file_head.set_cksum();
            blob.seek(std::io::SeekFrom::Start(0))?;
            tarball.append(&file_head, &mut blob.deref_mut())?;
        }
        tarball.finish()?;
        Ok(())
    }
}
