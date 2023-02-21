use std::path::Path;
use super::file::File;

pub struct Dependency {
    pub name: String,
}

impl Dependency {
    pub fn new(name: String) -> Dependency {
        Dependency { name }
    }

    pub fn is_exist_in(&self, path: &str) -> bool {
        Path::new(&format!("{}/{}", path, self.name)).exists()
    }

    pub fn get_file_from_path(&self, directories: &[String]) -> Option<File> {
        for dir in directories {
            if self.is_exist_in(dir) {
                let dependency_path = format!("{}/{}", dir, self.name);
                return Some(File::from_path(dependency_path).unwrap());
            }
        }
        return None;
    }
}