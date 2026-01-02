# Doppel

A fast duplicate file finder written in Rust.

## Features

- Scans directories recursively for duplicate files
- Uses SHA256 hashing for accurate duplicate detection
- Shows potential space savings

## Installation

```bash
cargo build --release
```

## Usage

```bash
doppel <path>
```

### Example

```bash
doppel "D:\Documents"
```

### Output

```
Duplicate group:
 - "D:\Documents\file1.txt"
 - "D:\Documents\backup\file1.txt"
Space savings if deleted: 1024 bytes
```

## How It Works

1. **Scan** - Walks through the directory and collects all files
2. **Hash** - Calculates SHA256 hash for each file
3. **Group** - Groups files by their hash value
4. **Report** - Displays duplicate groups and potential space savings

## Running Tests

```bash
cargo test
```

## Dependencies

- `clap` - Command line argument parsing
- `sha2` - SHA256 hashing
- `walkdir` - Recursive directory traversal

## License

MIT
