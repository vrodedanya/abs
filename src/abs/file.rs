use chrono::NaiveDateTime;

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
    last_modification: NaiveDateTime,
    ready_to_freeze: bool
}

#[allow(unused)]
impl File {
    pub fn new(path: String, last_modification: NaiveDateTime) -> File {
        File { path, last_modification, ready_to_freeze: false}
    }

    pub fn from_system_time(path: String, last_modification: std::time::SystemTime) -> File {
        File {path, last_modification: chrono::DateTime::<chrono::Local>::from(last_modification).naive_local(), ready_to_freeze: false}
    }

    pub fn from_path(path: String) -> Result<File, FileError> {
        let modified = std::path::Path::new(&path)
            .metadata().map_err(|err|FileError::CantGetMetaData(err.to_string()))?
            .modified().map_err(|err|FileError::ModificationTimeAnavailable(err.to_string()))?;
        Ok(File::from_system_time(path, modified))
    }

    pub fn mark_as_ready_to_freeze(&mut self) {
        self.ready_to_freeze = true;
    }

    pub fn path(&self) -> String {
        self.path.clone()
    }

    pub fn modified(&self) -> NaiveDateTime {
        self.last_modification
    }

    pub fn is_modified(&self, compare_time: NaiveDateTime) -> bool {
        self.last_modification.timestamp() != compare_time.timestamp()
    }

    pub fn modification_time_to_string(&self) -> String {
        self.last_modification.format("%Y-%m-%d/%T").to_string()
    }
}