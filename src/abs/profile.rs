use std::vec;

#[derive(Clone)]
#[allow(unused)]
pub struct Profile {
    pub name: String,
    pub compiler: String,
    pub standard: String,
    pub defines: Vec<String>,
    pub options: Vec<String>,
    pub linking_options: Vec<String>,
    pub linking_directories: Vec<String>,
    pub include_directories: Vec<String>,
}

#[allow(unused)]
impl Profile {
    pub fn new(name: &str) -> Profile {
        Profile {
            name: name.to_string(),
            compiler: String::new(),
            standard: String::new(),
            defines: vec![],
            options: vec![],
            linking_options: vec![],
            linking_directories: vec![],
            include_directories: vec![]
        }
    }
}