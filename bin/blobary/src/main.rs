// This is free and unencumbered software released into the public domain.

mod config;
mod hash;
mod input;
mod output;
mod store;
mod sync;
mod sysexits;

use crate::{
    hash::{encode_hash, parse_hash},
    input::{list_inputs, open_inputs, parse_bytesize},
    output::open_output,
    store::{open_store, open_store_from_url},
    sysexits::{exit, Sysexits},
};
use blobary::{
    Blob, BlobHash, BlobHasher, BlobStoreExt, IndexedBlobStoreIterator, DEFAULT_MIME_TYPE,
};
use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use shadow_rs::shadow;
use std::{
    io::{stdout, Read, Seek},
    ops::DerefMut,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use sync::copy_blobs;
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

    /// Enable verbose output
    #[clap(short = 'v', long, value_parser, global = true)]
    verbose: bool,

    /// Print version information
    #[clap(short = 'V', long, value_parser)]
    version: bool,

    /// Print version information
    #[clap(short = 'r', long, value_parser)]
    read_only: bool,

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
    Add {
        /// The input file(s) to add
        paths: Vec<PathBuf>,

        /// Split input into chunks of the given size
        #[clap(short = 'C', long, value_name = "SIZE", value_parser = parse_bytesize)]
        chunk: Option<usize>,
    },
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
        Commands::Add { paths, chunk } => match chunk {
            None => Commands::add(paths, &options),
            Some(chunk_size) => Commands::add_chunked(paths, chunk_size, &options),
        },
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
        print!("features: [");
        let mut features = Vec::from_iter(blobary::FEATURES.iter());
        #[cfg(feature = "7z")]
        features.push(&"7z");
        #[cfg(feature = "dmg")]
        features.push(&"dmg");
        #[cfg(feature = "tar")]
        features.push(&"tar");
        #[cfg(feature = "zip")]
        features.push(&"zip");
        features.sort();
        for (index, &&feature) in features.iter().enumerate() {
            if index > 0 {
                print!(", ");
            }
            print!("{}", feature);
        }
        println!("]");
        Ok(())
    }

    fn hash(input_paths: &Vec<impl AsRef<Path>>, _options: &Options) -> Result<(), Sysexits> {
        let input_paths = list_inputs(input_paths)?;
        for input_path in input_paths {
            let mut hasher = BlobHasher::new();
            if let Err(_err) = hasher.update_from_path(input_path) {
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
        let _store = open_store(false)?;
        Ok(()) // TODO
    }

    fn compact(options: &Options) -> Result<(), Sysexits> {
        let _store = open_store(!options.read_only)?;
        Ok(()) // TODO
    }

    fn list(options: &Options) -> Result<(), Sysexits> {
        let mut store = open_store(false)?;
        for blob in IndexedBlobStoreIterator::new(store.deref_mut()) {
            let blob_hash = encode_hash(blob.hash);
            if options.verbose || options.debug {
                let blob_data = blob.data.unwrap();
                let mut blob_data = blob_data.borrow_mut();
                let blob_type = blob_data.mime_type()?.unwrap_or(DEFAULT_MIME_TYPE);
                println!("{}\t{}\t{}", blob_hash, blob.size, blob_type);
            } else {
                println!("{}", blob_hash);
            }
        }
        Ok(())
    }

    fn add(input_paths: &Vec<impl AsRef<Path>>, options: &Options) -> Result<(), Sysexits> {
        let mut store = open_store(!options.read_only)?;
        let input_paths = list_inputs(input_paths)?;
        for input_path in input_paths {
            match store.put_file(input_path) {
                Err(err) => {
                    eprintln!("blobary: {}", err);
                    return Err(Sysexits::EX_IOERR);
                }
                Ok((created, blob)) => {
                    if created && (options.verbose || options.debug) {
                        println!("{}", encode_hash(blob.hash));
                    }
                }
            }
        }
        Ok(())
    }

    fn add_chunked(
        input_paths: &Vec<impl AsRef<Path>>,
        chunk_size: &usize,
        options: &Options,
    ) -> Result<(), Sysexits> {
        let mut store = open_store(!options.read_only)?;
        let input_paths = list_inputs(input_paths)?;

        let mut chunks = Vec::<Blob>::new();
        let mut buffer = Vec::<u8>::with_capacity(*chunk_size);
        buffer.resize(*chunk_size, 0);

        for input_path in input_paths {
            let input_file = &mut std::fs::File::open(input_path)?;
            loop {
                match input_file.read_exact(&mut buffer[..]) {
                    Ok(_) => {
                        let (_, blob) = store.put_bytes(&mut buffer[..])?;
                        if options.verbose || options.debug {
                            println!("{}", encode_hash(blob.hash));
                        }
                        assert_eq!(blob.size, *chunk_size as u64);
                        chunks.push(blob);
                    }
                    Err(err) if err.kind() == std::io::ErrorKind::UnexpectedEof => {
                        let input_offset = input_file.seek(std::io::SeekFrom::Current(0))?;
                        let chunk_size = input_offset as usize % buffer.len();
                        if chunk_size != 0 {
                            let (_, blob) = store.put_bytes(&buffer[0..chunk_size])?;
                            if options.verbose || options.debug {
                                println!("{}", encode_hash(blob.hash));
                            }
                            assert_eq!(blob.size, chunk_size as u64);
                            chunks.push(blob);
                        }
                        break;
                    }
                    Err(err) => {
                        eprintln!("blobary: {}", err);
                        return Err(Sysexits::EX_IOERR);
                    }
                }
            }
        }

        // if options.verbose || options.debug {
        //     for chunk in chunks {
        //         println!("{}", encode_hash(chunk.hash));
        //     }
        // }

        Ok(())
    }

    fn put(input_text: &String, options: &Options) -> Result<(), Sysexits> {
        let mut store = open_store(!options.read_only)?;
        match store.put_string(input_text) {
            Err(err) => {
                eprintln!("blobary: {}", err);
                return Err(Sysexits::EX_IOERR);
            }
            Ok((_created, blob)) => {
                if options.verbose || options.debug {
                    println!("{}", encode_hash(blob.hash));
                }
            }
        }
        Ok(())
    }

    fn get(blob_hashes: &Vec<BlobHash>, _options: &Options) -> Result<(), Sysexits> {
        let store = open_store(false)?;
        for blob_hash in blob_hashes {
            match store.get_by_hash(*blob_hash)? {
                None => return Err(Sysexits::EX_NOINPUT),
                Some(blob) => {
                    let blob_data = blob.data.unwrap();
                    let mut blob_data = blob_data.borrow_mut();
                    let mut stdout = stdout().lock();
                    std::io::copy(blob_data.deref_mut(), &mut stdout).unwrap();
                }
            }
        }
        Ok(())
    }

    fn remove(blob_hashes: &Vec<BlobHash>, options: &Options) -> Result<(), Sysexits> {
        let mut store = open_store(!options.read_only)?;
        for blob_hash in blob_hashes {
            store.remove(*blob_hash)?;
        }
        Ok(())
    }

    fn pull(remote_url: &Url, options: &Options) -> Result<(), Sysexits> {
        let mut source_store = open_store_from_url(remote_url, false)?;
        let mut target_store = open_store(!options.read_only)?;
        copy_blobs(&mut source_store, &mut target_store, options).map(|_| ())
    }

    fn push(remote_url: &Url, options: &Options) -> Result<(), Sysexits> {
        let mut source_store = open_store(false)?;
        let mut target_store = open_store_from_url(remote_url, !options.read_only)?;
        copy_blobs(&mut source_store, &mut target_store, options).map(|_| ())
    }

    fn sync(remote_url: &Url, options: &Options) -> Result<(), Sysexits> {
        // We download before uploading to minimize remote iteration overhead.
        let mut source_store = open_store_from_url(remote_url, !options.read_only)?;
        let mut target_store = open_store(!options.read_only)?;
        copy_blobs(&mut source_store, &mut target_store, options)
            .and_then(|_| copy_blobs(&mut target_store, &mut source_store, options))
            .map(|_| ())
    }

    fn import(input_paths: &Vec<impl AsRef<Path>>, options: &Options) -> Result<(), Sysexits> {
        let mut store = open_store(!options.read_only)?;
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
                        let (_, blob) = store.put(&mut file)?;
                        if file_hash != blob.hash {
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
        let mut store = open_store(false)?;
        let output = open_output(output_path)?;
        let blob_mtime: u64 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut tarball = tar::Builder::new(output);
        for blob in IndexedBlobStoreIterator::new(store.deref_mut()) {
            let blob_data = blob.data.unwrap();
            let mut blob_data = blob_data.borrow_mut();
            let mut file_head = Header::new_ustar();
            file_head.set_entry_type(EntryType::Regular);
            file_head.set_path(blob.hash.to_hex().as_str())?; // only base-16 supported here
            file_head.set_size(blob.size);
            file_head.set_mtime(blob_mtime);
            file_head.set_mode(0o444);
            file_head.set_username("root")?;
            file_head.set_groupname("root")?;
            file_head.set_cksum();
            blob_data.seek(std::io::SeekFrom::Start(0))?;
            tarball.append(&file_head, &mut blob_data.deref_mut())?;
        }
        tarball.finish()?;
        Ok(())
    }
}
