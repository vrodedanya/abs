use chrono::NaiveDateTime;

use std::{fs, path::Path};
use super::dependency::Dependency;
use super::section::RESULT_BORDER_WIDTH;
use colored::Colorize;


#[allow(unused)]
#[derive(Debug)]
pub enum FileError {
    CantGetMetadata(String),
    ModificationTimeUnavailable(String),
    FileDoesntExist(String),
    WrongPostfix(String)
}

#[derive(Hash, PartialEq, PartialOrd, Eq, Debug, Clone)]
pub struct File {
    pub path: String,
    last_modification: NaiveDateTime,
}

#[allow(unused)]
impl File {
    pub fn new(path: &str) -> Result<File, FileError> {
        let absolute_path = Path::new(path)
            .canonicalize()
            .map_err(|err| FileError::FileDoesntExist("Can't get absolute path".to_string()))?;
        let as_str = absolute_path.to_str().ok_or(FileError::FileDoesntExist("Can't find file".to_string()))?;

        let last_modification = absolute_path
            .metadata()
            .map_err(|err| FileError::CantGetMetadata(err.to_string()))?
            .modified()
            .map_err(|err| FileError::ModificationTimeUnavailable(err.to_string()))?;
        Ok(File {
            path: path.to_string(),
            last_modification: chrono::DateTime::<chrono::Local>::from(last_modification)
                .naive_local(),
        })
    }

    fn get_frozen_time_in(&self, subdirectory: &str) -> Option<NaiveDateTime> {
        let f = fs::File::open(self.get_freeze_path_in(subdirectory));
        if f.is_err() {
            return None;
        }
        let f = f.unwrap();
        let mut reader = std::io::BufReader::new(f);
        let mut content = String::new();
        if std::io::BufRead::read_line(&mut reader, &mut content).is_err() {
            return None;
        }
        if let Ok(time) = NaiveDateTime::parse_from_str(&content, "%Y-%m-%d/%T") {
            return Some(time);
        } else {
            return None;
        }
    }

    pub fn get_object_path_in(&self, subdirectory: &str) -> Result<String, FileError> {
        let without_extension = self.path
            .strip_suffix(".cpp")
            .or_else(|| self.path.strip_suffix(".c"))
            .ok_or(FileError::WrongPostfix("Neither C nor C++".to_string()))?;

        let file = File::encode_path(without_extension);

        Ok(format!(".abs/{subdirectory}/binary/{file}.o"))
    }

    pub fn get_freeze_path_in(&self, subdirectory: &str) -> String {
        let encoded_file = File::encode_path(&self.path);
        format!(".abs/{subdirectory}/frozen/{encoded_file}.frozen")
    }

    pub fn collect_dependencies(&self, search_list: &[String]) -> Vec<File> {
        let temp = Path::new(&self.path).canonicalize().unwrap();
        let path_to_file = temp.parent().unwrap().to_str().unwrap();
        let search_list: Vec<String> = search_list.to_vec().into_iter().chain(vec![path_to_file.to_string()]).collect();

        let file = fs::File::open(&self.path).unwrap();
        let reader = std::io::BufReader::new(file);

        std::io::BufRead::lines(reader)
            .map(|line| line.unwrap())
            .filter(|line| line.starts_with("#include"))
            .map(|line| {
                let stripped = line.strip_prefix("#include ").unwrap().trim();
                let name = stripped[1..stripped.len() - 1].to_string();
                let dependency = Dependency::new(name);

                if let Some(val) = dependency.get_file_from_path(&search_list) {
                    return val;
                } else {
                    println!(
                        "{:>RESULT_BORDER_WIDTH$} {}",
                        "Failed to find ".bright_red(),
                        dependency.name
                    );
                    std::process::exit(1);
                }
            })
            .collect()
    }

    pub fn is_modified_in(&self, subdirectory: &str) -> bool {
        match self.get_frozen_time_in(subdirectory) {
            Some(frozen_time) => self.last_modification.timestamp() > frozen_time.timestamp(),
            None => true,
        }
    }

    pub fn modification_time_to_string(&self) -> String {
        self.last_modification.format("%Y-%m-%d/%T").to_string()
    }
}

impl File {
    pub fn collect_files<const N: usize>(path: &str, suffixes: [&str; N]) -> Vec<File> {
        let mut vec_of_paths = vec![];
        for entry in std::fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let meta = entry.metadata().unwrap();
            let abs = entry.path().canonicalize().unwrap();
            let full_path = abs.to_str().unwrap();
            if meta.is_dir() {
                vec_of_paths.append(&mut File::collect_files(full_path, suffixes));
            } else if suffixes.iter().any(|&suffix| full_path.ends_with(suffix)) {
                vec_of_paths.push(File::new(full_path).unwrap());
            }
        }
        return vec_of_paths;
    }

    pub fn encode_path(path: &str) -> String {
        let mut result = String::new();
        let mut temp = path;
        loop {
            if let Some(distance) = temp.find('/') {
                result += &distance.to_string();
                if distance != 0 {
                    result += &temp[0..distance];
                }
                temp = &temp[(distance + 1)..];
            } else {
                if !temp.is_empty() {
                    result += &temp.len().to_string();
                    result += &temp;
                }
                break;
            }
        }
        return result;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn encoding_path() {
        assert_eq!(File::encode_path("/home"), "04home");
        assert_eq!(File::encode_path("home"), "4home");
        assert_eq!(File::encode_path("/home/user/dir/projects"), "04home4user3dir8projects");
        assert_eq!(File::encode_path("/test/"), "04test");
    }
}
