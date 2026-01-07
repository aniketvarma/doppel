use clap::Parser;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io::{BufReader, ErrorKind, Read};
use std::path::PathBuf;

use walkdir::WalkDir;

#[derive(Parser)]
// tnis struct contain all the users arguments
struct Args {
    /// path to scan for duplicates files
    path: PathBuf,

    #[arg(short, long)]
    /// delete duplicate files after listing them
    delete: bool,
}

#[derive(Debug)]
// struct containing each file's path, size and hash
struct FileData {
    path: PathBuf,
    size: u64,
    hash: Option<[u8; 32]>,
}

fn main() {
    let arg: Args = Args::parse();

    let files = file_fetcher(&arg.path); // fetch all the files in the given path

    if files.is_empty() {
        println!("No files found in {:?}", arg.path);
        return;
    }

    // generate hash for each file and insert into the struct
    let files_with_hash = calculate_hash(files);

    let duplicates = fetch_duplicate(files_with_hash);

    if arg.delete {
        delete_duplicates(&duplicates);
    } else {
        print_duplicates(&duplicates);
    }
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

fn fetch_duplicate(file_list: Vec<FileData>) -> Vec<Vec<FileData>> {
    // group is a hashmap where the key is the hash and value is a vector of FileData structs
    let mut groups: HashMap<[u8; 32], Vec<FileData>> = HashMap::new();

    // push each file into the hashmap based on its hash
    for file in file_list {
        if let Some(hash) = file.hash {
            groups.entry(hash).or_insert_with(Vec::new).push(file);
        }
    }

    // filter out the groups that have only one file (no duplicates)
    let duplicates: Vec<([u8; 32], Vec<FileData>)> = groups
        .into_iter()
        .filter(|(_, files)| files.len() > 1)
        .collect();

    // collecting only the groups of vectors of FileData structs in a new vector
    return duplicates
        .into_iter()
        .map(|(_hash, files_group)| files_group)
        .collect();
}

fn delete_duplicates(duplicates: &Vec<Vec<FileData>>) {

    let mut attempted = 0;
    let mut total_deleted = 0;
    let mut saved_space: u64 = 0;
    for duplicate in duplicates {
        for file in duplicate.iter().skip(1) {
            attempted += 1;
            total_deleted += 1;
            saved_space+= file.size;
            if let Err(e) = fs::remove_file(&file.path) {
                total_deleted -=1;
                saved_space-= file.size;
                match e.kind() {
                    ErrorKind::NotFound => println!("File already deleted, {}", file.path.display()),

                    ErrorKind::PermissionDenied => {
                        println!("Need Elevated Permission , {}", file.path.display())
                    }

                    _ => eprint!("{}", e),
                }
            }
        }
    }
    println!("Total files attempted for deletion: {}", attempted );
    println!("Total files deleted: {}" , total_deleted);
    println!("Total space saved:{}", readable_size(saved_space));
}

fn print_duplicates(duplicates: &Vec<Vec<FileData>>) {
    for duplicate_group in duplicates {
        let mut size: u64 = 0;
        println!("Duplicate group:");
        for file in duplicate_group {
            println!(" -> {}", file.path.display());
          size+= file.size;
        }
        size -= duplicate_group[0].size;
        println!("{} can be saved", readable_size(size));
        println!("Each file is of {}" , readable_size(duplicate_group[0].size));
        println!();
    }
}

fn readable_size(bytes: u64) -> String{
 const KB:u64 = 1024;
 const MB:u64 = KB*1024;
 const GB:u64 = MB*1024;

 if bytes >= GB {
    format!("{:.}GB", bytes as f64/ GB as f64)
 }else if bytes >= MB {
    format!("{:.}MB", bytes as f64/MB as f64)
 }else if bytes >= KB {
    format!("{:.}KB", bytes as f64/KB as f64)
 }else {
    format!("{}", bytes)
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
        File::create(&file_path)
            .unwrap()
            .write_all(b"hello")
            .unwrap();

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
        File::create(&file_path)
            .unwrap()
            .write_all(b"hello world")
            .unwrap();

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

        File::create(&file1)
            .unwrap()
            .write_all(b"same content")
            .unwrap();
        File::create(&file2)
            .unwrap()
            .write_all(b"same content")
            .unwrap();

        let files = vec![
            FileData {
                path: file1,
                size: 12,
                hash: None,
            },
            FileData {
                path: file2,
                size: 12,
                hash: None,
            },
        ];

        let result = calculate_hash(files);

        assert_eq!(result[0].hash, result[1].hash);
    }

    #[test]
    fn test_different_content_produces_different_hash() {
        let dir = tempdir().unwrap();

        let file1 = dir.path().join("file1.txt");
        let file2 = dir.path().join("file2.txt");

        File::create(&file1)
            .unwrap()
            .write_all(b"content A")
            .unwrap();
        File::create(&file2)
            .unwrap()
            .write_all(b"content B")
            .unwrap();

        let files = vec![
            FileData {
                path: file1,
                size: 9,
                hash: None,
            },
            FileData {
                path: file2,
                size: 9,
                hash: None,
            },
        ];

        let result = calculate_hash(files);

        assert_ne!(result[0].hash, result[1].hash);
    }
}
