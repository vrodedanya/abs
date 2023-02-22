use chrono::NaiveDateTime;

use std::{fs, path::Path, time::SystemTime};
use super::profile::Profile;
use super::dependency::Dependency;
use super::section::RESULT_BORDER_WIDTH;
use colored::Colorize;


#[allow(unused)]
#[derive(Debug)]
pub enum FileError {
    CantGetMetaData(String),
    ModificationTimeUnavailable(String),
    FileDoesntExist(String),
}

#[derive(Hash, PartialEq, PartialOrd, Eq, Debug, Clone)]
pub struct File {
    pub path: String,
    last_modification: NaiveDateTime,
}

#[allow(unused)]
impl File {
    pub fn new(path: &str, last_modification: NaiveDateTime) -> File {
        File {
            path: path.to_string(),
            last_modification,
        }
    }

    pub fn from_system_time(
        path: String,
        last_modification: SystemTime,
    ) -> Result<File, FileError> {
        let binding = Path::new(&path)
            .canonicalize()
            .map_err(|err| FileError::FileDoesntExist("Can't get absolute path".to_string()))?;
        let path = binding
            .to_str()
            .ok_or(FileError::FileDoesntExist("Can't find file".to_string()))?;
        Ok(File {
            path: path.to_string(),
            last_modification: chrono::DateTime::<chrono::Local>::from(last_modification)
                .naive_local(),
        })
    }

    pub fn from_path(path: String) -> Result<File, FileError> {
        let modified = Path::new(&path)
            .metadata()
            .map_err(|err| FileError::CantGetMetaData(err.to_string()))?
            .modified()
            .map_err(|err| FileError::ModificationTimeUnavailable(err.to_string()))?;
        Ok(File::from_system_time(path, modified)?)
    }

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
                vec_of_paths.push(File::from_path(full_path.to_owned()).unwrap());
            }
        }
        return vec_of_paths;
    }

    fn get_frozen_time(&self, section_name: &str, profile: &Profile) -> Option<NaiveDateTime> {
        if let Ok(f) = fs::File::open(self.get_freeze_path(section_name, profile)) {
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
        } else {
            return None;
        }
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

    pub fn get_object_path(&self, section_name: &str, profile: &Profile) -> String {
        // todo return error
        let without_extension = self.path
            .strip_suffix(".cpp")
            .or_else(|| self.path.strip_suffix(".c"))
            .expect("Expected cpp or c file");

        format!(
            ".abs/{}/{}/binary/{}",
            section_name,
            profile.name, 
            File::encode_path(&format!("{}{}", without_extension, ".o"))
        )
    }

    pub fn get_freeze_path(&self, section_name: &str, profile: &Profile) -> String {
        format!(
            ".abs/{}/{}/frozen/{}",
            section_name,
            profile.name,
            File::encode_path(&self.path)
        )
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

    pub fn modified(&self) -> NaiveDateTime {
        self.last_modification
    }

    pub fn is_modified(&self, section_name: &str, profile: &Profile) -> bool {
        match self.get_frozen_time(section_name, profile) {
            Some(frozen_time) => self.last_modification.timestamp() > frozen_time.timestamp(),
            None => true,
        }
        
    }

    pub fn modification_time_to_string(&self) -> String {
        self.last_modification.format("%Y-%m-%d/%T").to_string()
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;
    #[test]
    fn encoding_path() {
        assert_eq!(File::encode_path("/home"), "04home");
        assert_eq!(File::encode_path("home"), "4home");
        assert_eq!(File::encode_path("/home/user/dir/projects"), "04home4user3dir8projects");
        assert_eq!(File::encode_path("/test/"), "04test");
    }

    #[test]
    fn object_path() {
        let f = File::new("/some/test/path.cpp", Utc::now().naive_utc());
        let prof = Profile::empty("profile");

        assert_eq!(f.get_object_path("test", &prof), ".abs/test/profile/binary/04some4test6path.o");
        assert_eq!(f.get_object_path("otherSection", &prof), ".abs/otherSection/profile/binary/04some4test6path.o");
        assert_eq!(f.get_object_path("test", &prof), ".abs/test/profile/binary/04some4test6path.o");
    }

    #[test]
    fn freeze_path() {
        let f = File::new("/some/test/path.cpp", Utc::now().naive_utc());
        let prof = Profile::empty("profile");

        assert_eq!(f.get_freeze_path("test", &prof), ".abs/test/profile/frozen/04some4test8path.cpp");
        assert_eq!(f.get_freeze_path("otherSection", &prof), ".abs/otherSection/profile/frozen/04some4test8path.cpp");
        assert_eq!(f.get_freeze_path("test", &prof), ".abs/test/profile/frozen/04some4test8path.cpp");
    }
}
