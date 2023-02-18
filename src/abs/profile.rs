use std::vec;

#[derive(Debug)]
pub enum ProfileError {
    WrongType(String),
}

#[derive(Clone)]
#[derive(Debug)]
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
    pub fn empty(name: &str) -> Profile {
        Profile {
            name: name.to_string(),
            compiler: String::from("gcc"),
            standard: String::from("-std=c++17"),
            defines: vec![],
            options: vec![],
            linking_options: vec![],
            linking_directories: vec![],
            include_directories: vec![],
        }
    }

    pub fn from_config(name: &str, config: &toml::Value) -> Result<Profile, ProfileError> {
        let mut profile = Profile::empty(name);

        if let Some(compiler) = config.get("compiler") {
            profile.compiler = compiler
                .as_str()
                .ok_or_else(|| ProfileError::WrongType("compiler is string".to_string()))?
                .to_string();
        }
        if let Some(standard) = config.get("standard") {
            profile.standard = standard
                .as_str()
                .ok_or_else(|| ProfileError::WrongType("standard is string".to_string()))?
                .to_string();
        }
        if let Some(defines) = config.get("defines") {
            if defines.is_array() {
                let mut defines = defines.as_array().unwrap();
                if defines.iter().any(|elem| !elem.is_str()) {
                    return Err(ProfileError::WrongType(
                        "defines array can contain only string type".to_string(),
                    ));
                }
                let mut defines: Vec<String> = defines
                    .iter()
                    .map(|elem| elem.as_str().unwrap().to_string())
                    .collect();
                profile.defines.append(&mut defines);
            } else if defines.is_str() {
                let defines = defines.as_str().unwrap();
                profile.defines.push(defines.to_string());
            } else {
                return Err(ProfileError::WrongType(
                    "defines can be only an array or a string".to_string(),
                ));
            }
        }
        if let Some(options) = config.get("options") {
            if options.is_array() {
                let mut options = options.as_array().unwrap();
                if options.iter().any(|elem| !elem.is_str()) {
                    return Err(ProfileError::WrongType(
                        "options array can contain only string type".to_string(),
                    ));
                }
                let mut options: Vec<String> = options
                    .iter()
                    .map(|elem| elem.as_str().unwrap().to_string())
                    .collect();
                profile.options.append(&mut options);
            } else if options.is_str() {
                let options = options.as_str().unwrap();
                profile.options.push(options.to_string());
            } else {
                return Err(ProfileError::WrongType(
                    "options can be only an array or a string".to_string(),
                ));
            }
        }
        if let Some(linking_options) = config.get("linking_options") {
            if linking_options.is_array() {
                let mut linking_options = linking_options.as_array().unwrap();
                if linking_options.iter().any(|elem| !elem.is_str()) {
                    return Err(ProfileError::WrongType(
                        "linking options array can contain only string type".to_string(),
                    ));
                }
                let mut linking_options: Vec<String> = linking_options
                    .iter()
                    .map(|elem| elem.as_str().unwrap().to_string())
                    .collect();
                profile.linking_options.append(&mut linking_options);
            } else if linking_options.is_str() {
                let linking_options = linking_options.as_str().unwrap();
                profile.linking_options.push(linking_options.to_string());
            } else {
                return Err(ProfileError::WrongType(
                    "linking options can be only an array or a string".to_string(),
                ));
            }
        }
        if let Some(linking_directories) = config.get("linking_directories") {
            if linking_directories.is_array() {
                let mut linking_directories = linking_directories.as_array().unwrap();
                if linking_directories.iter().any(|elem| !elem.is_str()) {
                    return Err(ProfileError::WrongType(
                        "linking directories array can contain only string type".to_string(),
                    ));
                }
                let mut linking_directories: Vec<String> = linking_directories
                    .iter()
                    .map(|elem| elem.as_str().unwrap().to_string())
                    .collect();
                profile.linking_directories.append(&mut linking_directories);
            } else if linking_directories.is_str() {
                let linking_directories = linking_directories.as_str().unwrap();
                profile
                    .linking_directories
                    .push(linking_directories.to_string());
            } else {
                return Err(ProfileError::WrongType(
                    "linking directories can be only an array or a string".to_string(),
                ));
            }
        }
        if let Some(include_directories) = config.get("include_directories") {
            if include_directories.is_array() {
                let mut include_directories = include_directories.as_array().unwrap();
                if include_directories.iter().any(|elem| !elem.is_str()) {
                    return Err(ProfileError::WrongType(
                        "include directories array can contain only string type".to_string(),
                    ));
                }
                let mut include_directories: Vec<String> = include_directories
                    .iter()
                    .map(|elem| elem.as_str().unwrap().to_string())
                    .collect();
                profile.include_directories.append(&mut include_directories);
            } else if include_directories.is_str() {
                let include_directories = include_directories.as_str().unwrap();
                profile
                    .include_directories
                    .push(include_directories.to_string());
            } else {
                return Err(ProfileError::WrongType(
                    "include directories can be only an array or a string".to_string(),
                ));
            }
        }
        return Ok(profile);
    }
    
    pub fn fill_from_config(&mut self, config: &toml::Value) -> Result<(), ProfileError> {
        if let Some(compiler) = config.get("compiler") {
            self.compiler = compiler
                .as_str()
                .ok_or_else(|| ProfileError::WrongType("compiler is string".to_string()))?
                .to_string();
        }
        if let Some(standard) = config.get("standard") {
            self.standard = standard
                .as_str()
                .ok_or_else(|| ProfileError::WrongType("standard is string".to_string()))?
                .to_string();
        }
        if let Some(defines) = config.get("defines") {
            if defines.is_array() {
                let mut defines = defines.as_array().unwrap();
                if defines.iter().any(|elem| !elem.is_str()) {
                    return Err(ProfileError::WrongType(
                        "defines array can contain only string type".to_string(),
                    ));
                }
                let mut defines: Vec<String> = defines
                    .iter()
                    .map(|elem| elem.as_str().unwrap().to_string())
                    .collect();
                self.defines.append(&mut defines);
            } else if defines.is_str() {
                let defines = defines.as_str().unwrap();
                self.defines.push(defines.to_string());
            } else {
                return Err(ProfileError::WrongType(
                    "defines can be only an array or a string".to_string(),
                ));
            }
        }
        if let Some(options) = config.get("options") {
            if options.is_array() {
                let mut options = options.as_array().unwrap();
                if options.iter().any(|elem| !elem.is_str()) {
                    return Err(ProfileError::WrongType(
                        "options array can contain only string type".to_string(),
                    ));
                }
                let mut options: Vec<String> = options
                    .iter()
                    .map(|elem| elem.as_str().unwrap().to_string())
                    .collect();
                self.options.append(&mut options);
            } else if options.is_str() {
                let options = options.as_str().unwrap();
                self.options.push(options.to_string());
            } else {
                return Err(ProfileError::WrongType(
                    "options can be only an array or a string".to_string(),
                ));
            }
        }
        if let Some(linking_options) = config.get("linking_options") {
            if linking_options.is_array() {
                let mut linking_options = linking_options.as_array().unwrap();
                if linking_options.iter().any(|elem| !elem.is_str()) {
                    return Err(ProfileError::WrongType(
                        "linking options array can contain only string type".to_string(),
                    ));
                }
                let mut linking_options: Vec<String> = linking_options
                    .iter()
                    .map(|elem| elem.as_str().unwrap().to_string())
                    .collect();
                self.linking_options.append(&mut linking_options);
            } else if linking_options.is_str() {
                let linking_options = linking_options.as_str().unwrap();
                self.linking_options.push(linking_options.to_string());
            } else {
                return Err(ProfileError::WrongType(
                    "linking options can be only an array or a string".to_string(),
                ));
            }
        }
        if let Some(linking_directories) = config.get("linking_directories") {
            if linking_directories.is_array() {
                let mut linking_directories = linking_directories.as_array().unwrap();
                if linking_directories.iter().any(|elem| !elem.is_str()) {
                    return Err(ProfileError::WrongType(
                        "linking directories array can contain only string type".to_string(),
                    ));
                }
                let mut linking_directories: Vec<String> = linking_directories
                    .iter()
                    .map(|elem| elem.as_str().unwrap().to_string())
                    .collect();
                self.linking_directories.append(&mut linking_directories);
            } else if linking_directories.is_str() {
                let linking_directories = linking_directories.as_str().unwrap();
                self.linking_directories
                    .push(linking_directories.to_string());
            } else {
                return Err(ProfileError::WrongType(
                    "linking directories can be only an array or a string".to_string(),
                ));
            }
        }
        if let Some(include_directories) = config.get("include_directories") {
            if include_directories.is_array() {
                let mut include_directories = include_directories.as_array().unwrap();
                if include_directories.iter().any(|elem| !elem.is_str()) {
                    return Err(ProfileError::WrongType(
                        "include directories array can contain only string type".to_string(),
                    ));
                }
                let mut include_directories: Vec<String> = include_directories
                    .iter()
                    .map(|elem| elem.as_str().unwrap().to_string())
                    .collect();
                self.include_directories.append(&mut include_directories);
            } else if include_directories.is_str() {
                let include_directories = include_directories.as_str().unwrap();
                self.include_directories
                    .push(include_directories.to_string());
            } else {
                return Err(ProfileError::WrongType(
                    "include directories can be only an array or a string".to_string(),
                ));
            }
        };
        return Ok(());
    }
}
