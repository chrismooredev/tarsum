# tarsum
Reads an archive file and prints the contained file's hashes

Supports file hashes that are supported by the [checksums](https://crates.io/crates/checksums) crate.

Currently supported archive formats:
* zip
* tar
* tar.gz
* tar.xz
* tar.bz2
* PRs for more types are welcome!

Current help test:
```
USAGE:
    tarsum [FLAGS] [OPTIONS] [target]

ARGS:
    <target>
            If no file is provided, it is read over stdin, and --format must be supplied

FLAGS:
    -e, --escaped
            Perform C-style string escaping on filenames. Default is to force filenames to UTF8,
            which may be lossy.

        --help
            Prints help information

    -l, --lower
            Emit lowercase file hashes

    -V, --version
            Prints version information


OPTIONS:
    -f, --format <format>
            Read the file as a specific file format. Overrides the target's extension.

    -h, --hash <hash>
            Specify the hash algorithm [default: MD5]

    -r, --regex <regex>
            Outputs and hashes only matching filenames


Supported hash algorithms: (from the `checksums` rust crate, https://crates.io/crates/checksums)
    SHA1
    SHA2 = SHA2-512, SHA2-384, SHA2-256, SHA2-224
    SHA3 = SHA3-512, SHA3-256
    blake, blake2
    crc64, crc32, crc32c, crc16, crc8,
    md5, md6-128, md6-256, md6-512
    xor8
```
