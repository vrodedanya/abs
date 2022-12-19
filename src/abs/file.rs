use chrono::NaiveDateTime;

use super::profile::Profile;

#[allow(unused)]
#[derive(Debug)]
pub enum FileError {
    CantGetMetaData(String),
    ModificationTimeAnavailable(String),
    FileDoesntExist(String),
}

#[derive(Hash, PartialEq, PartialOrd, Eq, Debug, Clone)]
pub struct File {
    path: String,
    last_modification: NaiveDateTime
}

#[allow(unused)]
impl File {
    pub fn new(path: String, last_modification: NaiveDateTime) -> File {
        File { path, last_modification}
    }

    pub fn from_system_time(path: String, last_modification: std::time::SystemTime) -> Result<File, FileError>  {
        let binding = std::path::Path::new(&path).canonicalize()
            .map_err(|err|FileError::FileDoesntExist("Can't get absolute path".to_string()))?;
        let path = binding.to_str()
            .ok_or(FileError::FileDoesntExist("Can't find file".to_string()))?;
        Ok(File {path: path.to_string(), last_modification: chrono::DateTime::<chrono::Local>::from(last_modification).naive_local()})
    }

    pub fn from_path(path: String) -> Result<File, FileError> {
        let modified = std::path::Path::new(&path)
            .metadata().map_err(|err|FileError::CantGetMetaData(err.to_string()))?
            .modified().map_err(|err|FileError::ModificationTimeAnavailable(err.to_string()))?;
        Ok(File::from_system_time(path, modified)?)
    }

    pub fn path(&self) -> String {
        self.path.clone()
    }

    pub fn get_object_path(&self, section_name: &String, profile: &Profile) -> String {
        let file_name = self.path().replace("/", "|");
        let without_extension = file_name.strip_suffix(".cpp").or_else(||file_name.strip_suffix(".c"))
            .expect("Expected cpp or c file");

        format!(".abs/{}/{}/binary/{}{}", section_name, profile.name, without_extension, ".o")
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