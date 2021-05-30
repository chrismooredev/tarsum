
use std::{borrow::Cow, fs::File};
use std::io;
use std::io::Read;
use std::str::FromStr;
use std::path::{PathBuf};
use clap::Clap;
use checksums::{Algorithm, hash_reader};
use regex::bytes::Regex;
use flate2::read::GzDecoder;
use xz2::read::XzDecoder;
use bzip2::read::BzDecoder;
use zip::read::ZipFile;

// Algorithms scrapped from checksums::Algorithm::from_str
const HELP_SUFFIX: &'static str =
"Supported hash algorithms: (from the `checksums` rust crate, https://crates.io/crates/checksums)
    SHA1
    SHA2 = SHA2-512, SHA2-384, SHA2-256, SHA2-224
    SHA3 = SHA3-512, SHA3-256
    blake, blake2
    crc64, crc32, crc32c, crc16, crc8,
    md5, md6-128, md6-256, md6-512
    xor8
";

#[derive(Clap)]
#[clap(version = clap::crate_version!(), author = "Chris Moore", about = clap::crate_description!(), after_long_help = HELP_SUFFIX)]
pub struct Opts {
    /// Outputs and hashes only matching filenames
    #[clap(short, long)]
    regex: Option<Regex>,
    /// Specify the hash algorithm
    #[clap(short, long, default_value = "MD5")]
    hash: Algorithm,
    /// Emit lowercase file hashes
    #[clap(short, long)]
    lower: bool,
    /// Perform C-style string escaping on filenames. Default is to force filenames to UTF8, which may be lossy.
    #[clap(short, long)]
    escaped: bool,

    /// Read the file as a specific file format. Overrides the target's extension.
    #[clap(short, long)]
    format: Option<FileFormat>,

    /// If provided once, emits each file's size in bytes. If provided twice or more, emits in a human readable form (2.5MiB)
    #[clap(short, long, parse(from_occurrences))]
    size: u32,

    //#[clap(short = 'd', long, about = "Attempt to determine the decompression/archival format dynamically, instead of using the filename")]
    //content_detection: bool,

    // TODO: accept a basic format string? ala https://25thandclement.com/~william/projects/tarsum.html

    // only support one, because we would otherwise have to handle outputting where archives begin/end
    #[clap(about = "If no file is provided, it is read over stdin, and --format must be supplied")]
    target: Option<PathBuf>,
}
/*fn ioErrInvalidInput<E: Into<Box<dyn std::error::Error + Send + Sync>>>(s: E) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, s)
}*/
impl Opts {
    /// Uses the provided format if possible, otherwise parses the target filename's extension. Returns an Err if none is found, or a bad extension is parsed.
    fn format(&self) -> Result<FileFormat, String> {
        match &self.format {
            Some(fmt) => Ok(*fmt),
            None => match &self.target {
                None => Err("no provided file or file format".into()),
                Some(target) => {
                    //TODO: implement this in a way that doesn't require allocating

                    let basename = target.file_name().ok_or("target does not point to a file")?;
                    // the extension, at the very least, should appear as UTF8 - so we should be good
                    let basename_str = basename.to_string_lossy();

                    // extensions in reverse order ("gz" "tar")
                    let mut comps: Vec<&str> = basename_str.rsplit_terminator('.')
                        .take_while(|e| FILE_CONTAINERS.contains(e) || FILE_COMPRESSED.contains(e))
                        .collect();

                    // no found extensions
                    if comps.len() == 0 {
                        Err("target does not have any recognized extensions".into())
                    } else {
                        comps.reverse();
                        let ext = comps.join(".");
                        FileFormat::from_str(&ext)
                    }
                }
            }
        }
    }
}

/// Terminal file containers, that contain others
const FILE_CONTAINERS: &[&'static str] = &["zip", "tar"];
/// Compressed file formats, that decompress to another format
const FILE_COMPRESSED: &[&'static str] = &["gz", "xz", "bz2"];

// TODO: express each member as Member<Option<Box<FileFormat>>> to allow things like '.tar.gz.xz' instead of compound members like TarGz
#[derive(Debug, Copy, Clone)]
enum FileFormat {
    Zip,
    Tar,
    TarGz,
    TarXz,
    TarBz2,
    // iso
}
impl std::str::FromStr for FileFormat {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use FileFormat::*;
        
        Ok(match s.trim().to_lowercase().as_ref() {
            "zip" | "jar" => Zip,
            "tar" => Tar,
            "tar.gz" => TarGz,
            "tar.xz" => TarXz,
            "tar.bz2" => TarBz2,
            unk @ _ => Err(format!("`{}` is not a recognized archive type. See help for supported archives.", unk))?,
        })
    }
}
impl FileFormat {
    fn dump<R: Read>(&self, reader: R, opts: &Opts) -> io::Result<()> {
        use FileFormat::*;
        match self {
            Zip => dump::zip(reader, &opts),
            Tar => dump::tar(reader, &opts),
            TarGz => dump::tar(GzDecoder::new(reader), &opts),
            TarXz => dump::tar(XzDecoder::new(reader), &opts),
            TarBz2 => dump::tar(BzDecoder::new(reader), &opts),
        }
    }
}

fn main() -> io::Result<()> {
    let opts: Opts = Opts::parse();

    dump_archive(&opts)?;

    Ok(())
}

fn dump_archive(opts: &Opts) -> io::Result<()> {
    let fmt = opts.format()
        .map_err(|s| io::Error::new(io::ErrorKind::InvalidInput, s))?;

    // provided file, or stdin
    // getting the format asserts a provided file or --format flag
    if let Some(target) = &opts.target {
        let opened = File::open(target)?;
        fmt.dump(opened, opts)
    } else {
        let stdin = std::io::stdin();
        let stdinl = stdin.lock();
        fmt.dump(stdinl, opts)
    }
}

trait ArchiveFile: Read {
    // Note: using Vec<u8> to allow non-utf8 filenames in archives
    fn path<'a>(&'a self) -> Cow<'a, [u8]>;
    /// Returns if the path matches the regex. Should be implemented if `path` must be allocated be returned from .path()
    fn path_matches(&self, regex: &Regex) -> bool {
        regex.is_match(self.path().as_ref())
    }

    fn size(&self) -> u64;
}
impl ArchiveFile for ZipFile<'_> {
    fn path<'a>(&'a self) -> Cow<'a, [u8]> {
        Cow::Borrowed(self.name_raw())
    }
    fn size(&self) -> u64 {
        ZipFile::size(self)
    }
}
impl<R: Read> ArchiveFile for tar::Entry<'_, R> {
    fn path<'a>(&'a self) -> Cow<'a, [u8]> {
        self.path_bytes()
    }
    fn size(&self) -> u64 {
        tar::Entry::size(self)
    }
}

mod dump {
    use std::io::{self, Read};
    use super::{Opts, print_file};

    pub fn zip<R: Read>(mut reader: R, opts: &Opts) -> io::Result<()> {
        //let mut arc = zip::ZipArchive::new(reader)?;
        
        loop {
            match zip::read::read_zipfile_from_stream(&mut reader)? {
                None => break,
                Some(mut file) => {
                    if ! file.is_dir() {
                        print_file(&mut file, opts)
                    }
                }
            }
        }

        Ok(())
    }

    pub fn tar<R: Read>(reader: R, opts: &Opts) -> io::Result<()> {
        let mut arc = tar::Archive::new(reader);

        for file in arc.entries()? {
            let mut file = file?;
            if file.header().entry_type() != tar::EntryType::Directory {
                print_file(&mut file, opts);
            }
        }

        Ok(())
    }
}

fn print_file<F: ArchiveFile>(mut reader: &mut F, opts: &Opts) {
    // if this file passes the regex, or there is no regex...
    if opts.regex.as_ref().map(|reg| reader.path_matches(reg)).unwrap_or(true) {
        let hash_str = {
            let mut hstr = hash_reader(&mut reader, opts.hash);
            if opts.lower { hstr.make_ascii_lowercase() };
            hstr
        };

        // get the name as lossy UTF8, or escaped ascii
        let name_raw: Cow<[u8]> = reader.path();
        let name: Cow<str> = if opts.escaped {
            let escaped = name_raw.iter().copied()
                .map(std::ascii::escape_default)
                .flatten()
                .collect::<Vec<_>>();
            
            // SAFETY: std::ascii::escape_default should only emit valid UTF8 characters
            Cow::Owned(unsafe { String::from_utf8_unchecked(escaped) })
        } else {
            String::from_utf8_lossy(&name_raw)
        };

        match opts.size {
            0 => println!("{} {}", name, hash_str),
            1 => println!("{} {} {}", name, hash_str, reader.size()),
            _ => println!("{} {} {}", name, hash_str, bytesize::ByteSize(reader.size()).to_string_as(true)),
        }
        
    }
}

