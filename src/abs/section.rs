use std::collections::HashMap;
use chrono::NaiveDateTime;
use super::file::File;

#[derive(Debug)]
pub enum SectionError {
    MandatoryLack(String),
    FieldTypeError(String),
}

#[allow(unused)]
#[derive(Debug)]
pub struct Section {
    name: String,
    files: Vec<File>,
    include_directories: Vec<String>,
    deps_src: HashMap<File, Vec<File>>,
    srcs_dep: HashMap<File, Vec<File>>
}

#[allow(unused)]
impl Section {
    pub fn new(name: String, config: &toml::Value) -> Result<Section, SectionError> {
        std::fs::create_dir_all(format!(".abs/{}", name));

        let mut section_files: Vec<File> = vec![];

        let source_dir = config.get("source")
            .ok_or(SectionError::MandatoryLack("'source' is mandatory field!".to_string()))?
            .as_str()
            .ok_or(SectionError::FieldTypeError("'source' is string type!".to_string()))?;

        section_files = Section::collect_files(source_dir, [".hpp", ".cpp", ".h", ".c"]);

        let include_dir = config.get("include");

        let mut include_directories = Section::collect_default_includes();
        include_directories.push(source_dir.to_string());

        if include_dir.is_some() {
            let include_dir = include_dir.unwrap().as_str()
                .ok_or(SectionError::FieldTypeError("'source' is string type!".to_string()))?;
            section_files.append(&mut Section::collect_files(include_dir, [".hpp", ".cpp", ".h", ".c"]));

            include_directories.push(include_dir.to_string());
        }

        let deps_src = 
            Section::create_map_dependecy_sources(&section_files, &include_directories);
        let srcs_dep = 
            Section::create_map_source_dependencies(&section_files, &include_directories);

        Ok(Section {
            name, 
            files: section_files, 
            include_directories, 
            deps_src, 
            srcs_dep})
    }
    
    fn collect_files<const N: usize>(path: &str, suffixes: [&str; N]) -> Vec<File> {
        let mut vec_of_pathes = vec![];
        for entry in std::fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let meta = entry.metadata().unwrap();
            let abs = entry.path().canonicalize().unwrap();
            let full_path = abs.to_str().unwrap();
            if meta.is_dir() {
                vec_of_pathes.append(&mut Section::collect_files(full_path, suffixes));
            } else if suffixes.iter().any(|&suffix|full_path.ends_with(suffix)) {
                vec_of_pathes.push(File::from_system_time(full_path.to_owned(), meta.modified().unwrap()));
            }
        }
        return vec_of_pathes;
    }

    fn collect_default_includes() -> Vec<String> {
        let includes = std::process::Command::new("sh")
            .arg("-c")
            .arg("c++ -xc++ /dev/null -E -Wp,-v 2>&1 | sed -n 's,^ ,,p'").output().expect("failed to execute process");
    
        let includes = String::from_utf8(includes.stdout).unwrap();
        let includes: Vec<String> = includes.split("\n").map(str::to_string).filter(|str|!str.is_empty()).collect();
        includes
    }


    fn is_dependency_exist(dependency: &str, directory: &str) -> bool {
        return std::path::Path::new(&format!("{}/{}", directory, dependency)).exists();
    }

    fn get_dependency_path(dependency: &str, directories: &[String]) -> Option<File> {
        for dir in directories {
            if Section::is_dependency_exist(dependency, dir) {
                let dependency_path = format!("{}/{}", dir, dependency);
                return Some(File::from_path(dependency_path).unwrap());
            }
        }
        return None;
    }

    fn collect_local_dependencies_for_file(path: &str, search_list: &[String]) -> Vec<File> {
        let temp = std::path::Path::new(path).canonicalize().unwrap();
        let path_to_file = temp.parent().unwrap().to_str().unwrap();

        let file = std::fs::File::open(path).unwrap();
        let reader = std::io::BufReader::new(file);

        std::io::BufRead::lines(reader).map(|line|line.unwrap())
            .filter(|line| line.starts_with("#include"))
                .map(|line|{
            let stripped = line.strip_prefix("#include ").unwrap().trim();
            let name: String = stripped[1..stripped.len() - 1].to_string();
            // check local
            if Section::is_dependency_exist(&name, path_to_file) {
                let path = format!("{}/{}", path_to_file, name);
                return File::from_path(path).unwrap();
            }
            // check specialized pathes
            return Section::get_dependency_path(&name, search_list)
                .expect(&(String::from("Dependency doesn't exist: ") + &name));
        }).collect()
    }

    fn create_map_source_dependencies(pathes: &Vec<File>, search_list: &Vec<String>) -> HashMap<File, Vec<File>> {
        let mut map: HashMap<File, Vec<File>> = HashMap::new();
        for path in pathes {
            map.insert(path.to_owned(), Section::collect_local_dependencies_for_file(&path.path(), &search_list).into_iter().chain(vec![path.to_owned()].into_iter()).collect());
        }
        return map;
    }

    fn create_map_dependecy_sources(pathes: &Vec<File>, search_list: &Vec<String>) -> HashMap<File, Vec<File>> {
        let mut map: HashMap<File, Vec<File>> = HashMap::new();
        let src_dep = Section::create_map_source_dependencies(pathes, search_list);
        for (source, dependencies) in src_dep.iter() {
            for dependency in dependencies {
                if map.contains_key(dependency) {
                    map.get_mut(dependency).unwrap().push(source.to_owned());
                } else {
                    map.insert(dependency.to_owned(), vec![source.to_owned()]);
                }
            }
        }
        return map;
    }

    fn get_modified(&self, files: &Vec<File>) -> Vec<File> {
        let mut files = files.clone();
        files.retain(|file| {
            let f = std::fs::File::open(format!(".abs/{}/frozen/{}", self.name, file.path().replace("/", "|")));
            if f.is_err() {
                return true;
            }
            let f = f.unwrap();
            let mut f = std::io::BufReader::new(f);
            let mut content = String::new();
            let res = std::io::BufRead::read_line(&mut f, &mut content).expect("expect string in the file");

            let time = NaiveDateTime::parse_from_str(&content, "%Y-%m-%d/%T").unwrap().into();
            if file.is_modified(time) {
                return true;
            } else {
                return false;
            }
        });
        return files;
    }

    fn freeze(&self, file: &File) {
        std::fs::create_dir_all(format!(".abs/{}/frozen/", self.name));
        let f = std::fs::File::create(format!(".abs/{}/frozen/{}", self.name, file.path().replace("/", r"|")))
            .expect("Unable to create file");
        let mut f = std::io::BufWriter::new(f);
        std::io::Write::write(&mut f, file.modification_time_to_string().as_bytes()).expect("writted");
    }

}

#[allow(unused)]
impl Section {
    pub fn check(&self) -> bool {
        let mut is_successful = true;
        let mut built: Vec<String> = vec![];
        for (dep, srcs) in &self.deps_src {
            for src in srcs.iter() {
                if built.contains(&src.path()) || src.path().ends_with(".hpp") || src.path().ends_with(".h") {
                    continue;
                }
                let args = self.include_directories.iter().map(|str|format!("-I{}", str));
                let mut child = std::process::Command::new("g++")
                    .arg("-fsyntax-only")
                    .args(args)
                    .arg(&src.path())
                    .spawn().unwrap();
                match child.wait() {
                    Ok(exit_status) => {
                        if exit_status.success() {
                            println!("Complete: {}", src.path());
                        } else {
                            println!("Failed: {}", src.path());
                            is_successful = false;
                        }
                    },
                    Err(_) => println!("Failed: {}", src.path()),
                }
                built.push(src.path());
            }
        }
        return is_successful;
    }

    pub fn build(&self) -> bool {
        std::fs::create_dir_all(format!(".abs/{}/binary/", self.name));

        let mut is_successful = true;
        let mut built: Vec<String> = vec![];
        let mut objects: Vec<String> = vec![];

        let modified = self.get_modified(&self.deps_src.keys().cloned().collect());
        println!("{:#?}", modified);

        // todo add building based on dependency changing
        for (dep, srcs) in &self.deps_src {
            for src in srcs.iter() {
                if built.contains(&src.path()) || src.path().ends_with(".hpp") || src.path().ends_with(".h") {
                    continue;
                }
                let args = self.include_directories.iter().map(|str|format!("-I{}", str));
                let file_name = std::path::Path::new(&src.path()).file_name().unwrap()
                    .to_str().unwrap().to_string();
                let without_extension = file_name.strip_suffix(".cpp").or_else(||file_name.strip_suffix(".c"))
                    .expect("Expected cpp or c file");

                let output_name = format!(".abs/{}/binary/{}{}", self.name, without_extension, ".o");

                let mut child = std::process::Command::new("g++")
                    .arg("-c")
                    .args(args)
                    .arg(&src.path())
                    .arg("-o")
                    .arg(&output_name)
                    .spawn().unwrap();

                objects.push(output_name);

                match child.wait() {
                    Ok(exit_status) => {
                        if exit_status.success() {
                            println!("Complete: {}", src.path());
                            self.freeze(src);
                        } else {
                            println!("Failed: {}", src.path());
                            is_successful = false;
                        }
                    },
                    Err(_) => println!("Failed: {}", src.path()),
                }
                built.push(src.path());
            }
            self.freeze(dep);
        }
        if !is_successful {
            return false;
        }

        // linking
        let mut child = std::process::Command::new("g++")
            .args(objects)
            .arg("-o")
            .arg(format!(".abs/{}/binary/{}", self.name, self.name))
            .spawn().unwrap();
        return is_successful;
    }

    pub fn run(&self) -> bool{
        if !self.build() {
            return false;
        }
        println!("Program:");
        let mut child = std::process::Command::new(format!(".abs/{}/binary/{}", self.name, self.name))
            .spawn().unwrap();
        return true;
    }
}