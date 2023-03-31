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

    pub fn get_file_from_path(&self, directories_for_search: &[String]) -> Option<File> {
        for dir in directories_for_search {
            if self.is_exist_in(dir) {
                let dependency_path = format!("{}/{}", dir, self.name);
                return Some(File::new(&dependency_path).unwrap());
            }
        }
        return None;
    }
}