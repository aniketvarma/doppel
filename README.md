# Doppel

A fast duplicate file finder written in Rust that identifies and optionally deletes duplicate files based on SHA256 content hashing.

## Features

- **Recursive directory scanning** - Scans directories and subdirectories for all files
- **SHA256 content-based detection** - Identifies duplicates by comparing file content hashes, not just names
- **Smart grouping** - Groups identical files together
- **Human-readable output** - Displays file sizes in KB, MB, or GB format
- **Delete functionality** - Option to safely delete duplicate files with detailed error handling
- **Graceful error handling** - Handles permission errors and missing files without stopping the process
- **Comprehensive testing** - Unit tests for core functionality
- **Progress tracking** - Tracks attempted deletions vs. successful deletions

## Installation

```bash
cargo build --release
```

## Usage

### View duplicates (preview mode)

```bash
doppel "C:\path\to\directory"
```

### Delete duplicates

```bash
doppel "C:\path\to\directory" --delete
```

### Examples

**Find duplicates:**
```bash
doppel "D:\Documents"
```

**Delete duplicates:**
```bash
doppel "D:\Documents" --delete
```

## Output Examples

**Preview mode (without --delete):**
```
Duplicate group:
  -> D:\Documents\file1.txt (5.2 MB)
  -> D:\Documents\backup\file1.txt (5.2 MB)
Space savings if deleted: 5.2 MB

Duplicate group:
  -> D:\Pictures\photo.jpg (2.8 MB)
  -> D:\Archive\photo.jpg (2.8 MB)
Space savings if deleted: 2.8 MB
```

**Delete mode (with --delete):**
```
Deleted: D:\Documents\backup\file1.txt
Deleted: D:\Archive\photo.jpg

Total files attempted for deletion: 2
Total files deleted: 2
Space freed: 8.0 MB
```

## How It Works

1. **Scan** - Recursively walks through the directory and collects all files with their sizes
2. **Hash** - Calculates SHA256 hash for each file's content
3. **Group** - Uses a HashMap to group files by their hash value
4. **Filter** - Identifies groups with more than one file (actual duplicates)
5. **Report** - Displays duplicate groups with space savings information
6. **Delete (optional)** - If `--delete` flag is used, removes duplicate files while keeping one copy per group

## Error Handling

The tool gracefully handles various error scenarios:

- **File already deleted** - Logs and continues if a file was deleted between scanning and deletion
- **Permission denied** - Reports permission errors and continues processing other files
- **Other errors** - Catches unexpected errors and provides detailed information

## Running Tests

```bash
cargo test
```

The test suite includes:
- File scanning and collection tests
- Hash calculation verification
- Same content produces same hash validation
- Different content produces different hash validation
- Empty directory handling
- Empty file hashing

## Algorithm Details

**Duplicate Detection:** Files are considered duplicates if they have identical SHA256 hashes. SHA256 collisions are cryptographically infeasible, making this approach reliable for practical use.

**Space Savings Calculation:** When multiple files are identical, space savings are calculated as the total size minus the first file (the one that's kept).

## Dependencies

- `clap` - Command line argument parsing with derive macros
- `sha2` - SHA256 hashing algorithm
- `walkdir` - Recursive directory traversal
- `tempfile` - Temporary file creation for testing

## Technical Notes

- Files are processed in chunks (8192 bytes) for efficient memory usage with large files
- HashMap ensures O(1) lookup for duplicate detection
- All file operations use proper error handling with io::Error kind matching
- The tool maintains counters for attempted vs. successful deletions

## License

MIT
