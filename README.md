# dirhash
Compute SHA-256 for files in a directory and list duplicates.

## Build
```bash
cargo build --release
./target/release/dirhash ./data --exts jpg,png --dupes > dupes.csv
```
