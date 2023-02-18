use chrono::NaiveDateTime;

use super::profile::Profile;


#[allow(unused)]
#[derive(Debug)]
pub enum FileError {
    CantGetMetaData(String),
    ModificationTimeUnavailable(String),
    FileDoesntExist(String),
}

#[derive(Hash, PartialEq, PartialOrd, Eq, Debug, Clone)]
pub struct File {
    path: String,
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
        last_modification: std::time::SystemTime,
    ) -> Result<File, FileError> {
        let binding = std::path::Path::new(&path)
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
        let modified = std::path::Path::new(&path)
            .metadata()
            .map_err(|err| FileError::CantGetMetaData(err.to_string()))?
            .modified()
            .map_err(|err| FileError::ModificationTimeUnavailable(err.to_string()))?;
        Ok(File::from_system_time(path, modified)?)
    }

    pub fn path(&self) -> String {
        self.path.clone()
    }

    pub fn encode_path(path: &str) -> String {
        let mut result = String::new();
        let mut temp = path.clone();
        loop {
            let distance = temp.find('/');
            if distance.is_none() {
                if !temp.is_empty() {
                    result += &temp.len().to_string();
                    result += &temp;
                }
                break;
            }
            let distance = distance.unwrap();
            result += &distance.to_string();
            if distance != 0 {
                result += &temp[0..distance];
            }
            temp = &temp[distance + 1..];
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

    pub fn modified(&self) -> NaiveDateTime {
        self.last_modification
    }

    pub fn is_modified(&self, compare_time: &NaiveDateTime) -> bool {
        self.last_modification.timestamp() > compare_time.timestamp()
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

