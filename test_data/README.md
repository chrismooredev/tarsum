# Test data

Meant to hold test cases involving variations in file structure and directory contents.

All archives are created by putting the files inside `root` (but not `root` itself) into the archive.

Files randomly created with the linux command
```bash
dd if=/dev/urandom of=/dev/stdout bs=1KiB count=1 | xxd >$output.txt
```