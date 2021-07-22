# tarsum
[![crates.io](https://img.shields.io/crates/v/tarsum.svg)](https://crates.io/crates/tarsum) [![docs.rs](https://docs.rs/tarsum/badge.svg)](https://docs.rs/crate/tarsum/0.1.1)

Reads an archive file and prints hashes of the files within it.

Supports file hashes that are supported by the [checksums](https://crates.io/crates/checksums) crate.

Currently supported archive formats:
* zip/jar
* tar
* tar.gz
* tar.xz
* tar.bz2
* PRs for more formats are welcome!

### Todo:
* more archive types
* tests
* provide JSON/? output for easier more readable machine output

Current help text:
```
USAGE:
    tarsum [FLAGS] [OPTIONS] [target]

ARGS:
    <target>
            If no file is provided, it is read over stdin, and --format must be supplied

FLAGS:
    -e, --escaped
            Perform C-style string escaping on filenames. Default is to force filenames to UTF8,
            which may be lossy

        --help
            Prints help information

    -l, --lower
            Emit lowercase file hashes

    -s, --size
            If provided once, emits each file's size in bytes. If provided twice or more, emits in a
            human readable form (2.5MiB)

    -V, --version
            Prints version information


OPTIONS:
    -f, --format <format>
            Read the file as a specific file format. Overrides the target's extension

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

Example output:
```
tarsum$ tarsum ./test_data/root.tar
a/aa/f.txt 75FFCBE62A0C6E7C5E770A55BBA54C67
a/aa/g.txt 576248B3F051506BB5316CBAA90A0778
a/bb/h.txt C14E72CEE3A122DEE5B64D73EFD91B19
a/d.txt A91067CFE85BA92E6D9FD5538CD75117
a/e.txt CD12B29773CF3BFB72C2F6DDBF7AD0EC
a.txt 5E70D8E77DAF4B3E0CEB05E0868C6530
b/aa/i.txt 24F5874A3F293722FCD9FDFCDB967382
b.txt 2284615785EAAC08938CE4B1EAFF0E1E
c.txt 5C08CE6C57F9957003246E92D07D03D4
```
