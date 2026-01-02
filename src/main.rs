use clap::Parser;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use walkdir::WalkDir;
use std::fs;

#[derive(Parser)]
struct Args {
    /// path to scan for duplicates files
    path: PathBuf,
}

#[derive(Debug)]
struct FileData {
    path: PathBuf,
    size: u64,
    hash: Option<[u8; 32]>,
}

fn main() {
    let arg: Args = Args::parse();

    let files = file_fetcher(&arg.path);
    if files.is_empty() {
        println!("No files found in {:?}", arg.path);
        return;
    }

    let files_with_hash = calculate_hash(files);

    fetch_duplicate(files_with_hash);
}

fn file_fetcher(path: &PathBuf) -> Vec<FileData> {
    let mut file_list: Vec<FileData> = Vec::new();

    let files_iterator = WalkDir::new(path);

    for file in files_iterator {
        match file {
            Ok(entry) => {
                let metadata = entry.metadata().unwrap();

                if metadata.is_file() {
                    let file_data = FileData {
                        path: entry.path().to_path_buf(),
                        size: metadata.len(),
                        hash: None,
                    };

                    file_list.push(file_data);
                }
            }

            Err(e) => {
                eprintln!("Error reading file: {}", e);
            }
        }
    }

    file_list
}

fn calculate_hash(file_list: Vec<FileData>) -> Vec<FileData> {
    let iterator = file_list
        .into_iter()
        .map(|mut filedata| {
            let file = fs::File::open(&filedata.path).unwrap();
            let mut reader = BufReader::new(file);
            let mut buffer = [0u8; 8192];
            let mut hasher = Sha256::new();
            loop {
                let bytes_read = reader.read(&mut buffer).unwrap();
                if bytes_read == 0 {
                    break;
                }
                hasher.update(&buffer[..bytes_read]);
            }
            filedata.hash = Some(hasher.finalize().into());

            filedata
        })
        .collect();

    iterator
}

fn fetch_duplicate(file_list: Vec<FileData>) {
    let mut groups: HashMap<[u8; 32], Vec<FileData>> = HashMap::new();

    for file in file_list {
        if let Some(hash) = file.hash {
            groups.entry(hash).or_insert_with(Vec::new).push(file);
        } 
    }
    let duplicates: Vec<([u8; 32], Vec<FileData>)> = groups
            .into_iter()
            .filter(|(_, files)| files.len() > 1)
            .collect();

        let duplicates: Vec<Vec<FileData>> = duplicates
            .into_iter()
            .map(|(_hash, files_group)| files_group)
            .collect();

        for duplicate_group in &duplicates {
            println!("Duplicate group:");
            let mut total_size: u64 = 0;
            for file in duplicate_group {
                println!(" - {:?}", file.path);
                total_size += file.size;
            }
            println!("Space savings if deleted: {} bytes", total_size - duplicate_group[0].size);
            println!();

        }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_file_fetcher_finds_files() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        File::create(&file_path).unwrap().write_all(b"hello").unwrap();

        let files = file_fetcher(&dir.path().to_path_buf());

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].size, 5);
    }

    #[test]
    fn test_file_fetcher_ignores_directories() {
        let dir = tempdir().unwrap();
        fs::create_dir(dir.path().join("subdir")).unwrap();

        let files = file_fetcher(&dir.path().to_path_buf());

        assert_eq!(files.len(), 0);
    }

    #[test]
    fn test_calculate_hash_produces_hash() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        File::create(&file_path).unwrap().write_all(b"hello world").unwrap();

        let files = vec![FileData {
            path: file_path,
            size: 11,
            hash: None,
        }];

        let result = calculate_hash(files);

        assert_eq!(result.len(), 1);
        assert!(result[0].hash.is_some());
    }

    #[test]
    fn test_same_content_produces_same_hash() {
        let dir = tempdir().unwrap();
        
        let file1 = dir.path().join("file1.txt");
        let file2 = dir.path().join("file2.txt");
        
        File::create(&file1).unwrap().write_all(b"same content").unwrap();
        File::create(&file2).unwrap().write_all(b"same content").unwrap();

        let files = vec![
            FileData { path: file1, size: 12, hash: None },
            FileData { path: file2, size: 12, hash: None },
        ];

        let result = calculate_hash(files);

        assert_eq!(result[0].hash, result[1].hash);
    }

    #[test]
    fn test_different_content_produces_different_hash() {
        let dir = tempdir().unwrap();
        
        let file1 = dir.path().join("file1.txt");
        let file2 = dir.path().join("file2.txt");
        
        File::create(&file1).unwrap().write_all(b"content A").unwrap();
        File::create(&file2).unwrap().write_all(b"content B").unwrap();

        let files = vec![
            FileData { path: file1, size: 9, hash: None },
            FileData { path: file2, size: 9, hash: None },
        ];

        let result = calculate_hash(files);

        assert_ne!(result[0].hash, result[1].hash);
    }
}
