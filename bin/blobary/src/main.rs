// This is free and unencumbered software released into the public domain.

mod config;
mod hash;
mod input;
mod output;
mod store;
mod sysexits;

use crate::{
    hash::{encode_hash, parse_hash},
    input::{list_inputs, open_inputs},
    output::open_output,
    store::open_store,
    sysexits::{exit, Sysexits},
};
use blobary::{BlobHash, BlobHasher, BlobIterator, BlobStore, BlobStoreExt, DEFAULT_MIME_TYPE};
use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use shadow_rs::shadow;
use std::{
    io::stdout,
    ops::DerefMut,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use tar::{EntryType, Header};
use url::Url;

shadow!(build);

/// Blobary command-line interface (CLI)
#[derive(Parser, Debug)]
#[command(name = "Blobary", about)]
#[command(arg_required_else_help = true)]
struct Options {
    /// Enable debugging output
    #[clap(short = 'd', long, value_parser, global = true)]
    debug: bool,

    /// Show license information
    #[clap(long, value_parser)]
    license: bool,

    // Enable verbose output
    #[clap(short = 'v', long, value_parser, global = true)]
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
    Get {
        #[arg(value_parser = parse_hash)]
        ids: Vec<BlobHash>,
    },
    /// Remove a blob by its hash
    #[clap(aliases = &["rm", "del", "delete"])]
    Remove {
        #[arg(value_parser = parse_hash)]
        ids: Vec<BlobHash>,
    },
    /// Pull blobs from another repository
    Pull {
        #[arg(value_parser = Url::parse)]
        url: Url,
    },
    /// Push blobs to another repository
    Push {
        #[arg(value_parser = Url::parse)]
        url: Url,
    },
    /// Sync blobs with another repository
    Sync {
        #[arg(value_parser = Url::parse)]
        url: Url,
    },
    /// Import blobs from a tarball
    Import { paths: Vec<PathBuf> },
    /// Export blobs to a tarball
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

    fn hash(input_paths: &Vec<impl AsRef<Path>>, _options: &Options) -> Result<(), Sysexits> {
        let input_paths = list_inputs(input_paths)?;
        for input_path in input_paths {
            let mut hasher = BlobHasher::new();
            if let Err(_err) = hasher.update_mmap(input_path) {
                return Err(Sysexits::EX_IOERR);
            }
            let hash = hasher.finalize();
            println!("{}", encode_hash(hash));
        }
        Ok(())
    }

    fn init(_options: &Options) -> Result<(), Sysexits> {
        Ok(()) // TODO
    }

    fn check(_options: &Options) -> Result<(), Sysexits> {
        let _store = open_store()?;
        Ok(()) // TODO
    }

    fn compact(_options: &Options) -> Result<(), Sysexits> {
        let _store = open_store()?;
        Ok(()) // TODO
    }

    fn list(options: &Options) -> Result<(), Sysexits> {
        let mut store = open_store()?;
        for blob in BlobIterator::new(&mut store) {
            let mut blob = blob.borrow_mut();
            let blob_hash = blob.hash()?;
            let blob_hash = encode_hash(blob_hash);
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

    fn add(input_paths: &Vec<impl AsRef<Path>>, _options: &Options) -> Result<(), Sysexits> {
        let mut store = open_store()?;
        let input_paths = list_inputs(input_paths)?;
        for input_path in input_paths {
            let result = store.put_file(input_path);
            if let Err(_err) = result {
                return Err(Sysexits::EX_IOERR);
            }
            let blob_id = result.unwrap();
            let blob_hash = store.id_to_hash(blob_id).unwrap();
            println!("{}", encode_hash(blob_hash));
        }
        Ok(())
    }

    fn put(input_text: &String, _options: &Options) -> Result<(), Sysexits> {
        let mut store = open_store()?;
        let result = store.put_string(input_text);
        if let Err(_err) = result {
            return Err(Sysexits::EX_IOERR);
        }
        let blob_id = result.unwrap();
        let blob_hash = store.id_to_hash(blob_id).unwrap();
        println!("{}", encode_hash(blob_hash));
        Ok(())
    }

    fn get(blob_hashes: &Vec<BlobHash>, _options: &Options) -> Result<(), Sysexits> {
        let store = open_store()?;
        for blob_hash in blob_hashes {
            match store.get_by_hash(*blob_hash) {
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

    fn remove(blob_hashes: &Vec<BlobHash>, _options: &Options) -> Result<(), Sysexits> {
        let _store = open_store()?;
        for _blob_hash in blob_hashes {
            // TODO
        }
        Ok(())
    }

    fn pull(_remote_url: &Url, _options: &Options) -> Result<(), Sysexits> {
        let _store = open_store()?;
        Ok(()) // TODO
    }

    fn push(_remote_url: &Url, _options: &Options) -> Result<(), Sysexits> {
        let _store = open_store()?;
        Ok(()) // TODO
    }

    fn sync(_remote_url: &Url, _options: &Options) -> Result<(), Sysexits> {
        let _store = open_store()?;
        Ok(()) // TODO
    }

    fn import(input_paths: &Vec<impl AsRef<Path>>, _options: &Options) -> Result<(), Sysexits> {
        let mut store = open_store()?;
        let inputs = open_inputs(input_paths)?;
        for (input_path, input) in inputs {
            let mut tarball = tar::Archive::new(input);
            for file in tarball.entries()? {
                let mut file = file?;
                //let file_size = file.header().size()?;
                let file_path = file.path_bytes();
                // only base-16 supported here
                match BlobHash::from_hex(file_path) {
                    Ok(file_hash) => {
                        let blob_id = store.put(&mut file)?;
                        let blob_hash = store.id_to_hash(blob_id).unwrap();
                        if file_hash != blob_hash {
                            eprintln!("{}: hash mismatch in tarball", input_path);
                            return Err(Sysexits::EX_DATAERR);
                        }
                    }
                    Err(_err) => (),
                }
            }
        }
        Ok(())
    }

    fn export(output_path: &Option<impl AsRef<Path>>, _options: &Options) -> Result<(), Sysexits> {
        let mut store = open_store()?;
        let output = open_output(output_path)?;
        let blob_mtime: u64 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .try_into()
            .unwrap();
        let mut tarball = tar::Builder::new(output);
        for blob in BlobIterator::new(&mut store) {
            let mut blob = blob.borrow_mut();
            let blob_size = blob.size()?;
            let blob_hash = blob.hash()?;
            let mut file_head = Header::new_ustar();
            file_head.set_entry_type(EntryType::Regular);
            file_head.set_path(blob_hash.to_hex().as_str())?; // only base-16 supported here
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
