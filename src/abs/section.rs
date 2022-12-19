use std::{collections::HashMap, io::Read, process::Child};
use chrono::NaiveDateTime;
use colored::Colorize;
use super::{file::File, profile::Profile};

#[derive(Debug)]
pub enum SectionError {
    MandatoryLack(String),
    FieldTypeError(String),
}

#[allow(unused)]
#[derive(Debug)]
pub struct Section {
    pub name: String,
    files: Vec<File>,
    include_directories: Vec<String>,
    deps_src: HashMap<File, Vec<File>>,
    srcs_dep: HashMap<File, Vec<File>>
}

const RESULT_BORDED_WIDTH: usize = 10;

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
                vec_of_pathes.push(File::from_path(full_path.to_owned()).unwrap());
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
            map.insert(path.to_owned(), 
                Section::collect_local_dependencies_for_file(&path.path(), &search_list)
                    .into_iter().chain(vec![path.to_owned()].into_iter()).collect());
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

    fn get_frozen_time(&self, file: &File, profile: &Profile) -> Option<NaiveDateTime> {
        let f = std::fs::File::open(format!(".abs/{}/{}/frozen/{}", self.name, profile.name, file.path().replace("/", "|")));
        if f.is_err() {
            return None;
        }
        let mut reader = std::io::BufReader::new(f.unwrap());
        let mut content = String::new();
        let res = std::io::BufRead::read_line(&mut reader, &mut content).expect("expect string in the file");

        let time = NaiveDateTime::parse_from_str(&content, "%Y-%m-%d/%T");
        if time.is_err() {
            return None;
        }
        Some(time.unwrap())
    }

    fn get_modified(&self, files: &Vec<File>, profile: &Profile) -> Vec<File> {
        let mut files = files.clone();
        files.retain(|file| {
            match self.get_frozen_time(file, profile) {
                Some(frozen_time) => file.is_modified(&frozen_time),
                None => true
            }
        });
        return files;
    }

    fn freeze(&self, file: &File, profile: &Profile) {
        std::fs::create_dir_all(format!(".abs/{}/{}/frozen/", self.name, profile.name));
        let f = std::fs::File::create(format!(".abs/{}/{}/frozen/{}", self.name, profile.name, file.path().replace("/", r"|")))
            .expect("Unable to create file");
        let mut f = std::io::BufWriter::new(f);
        let file_changed = file.modified().format("%Y-%m-%d/%T").to_string();
        let now = chrono::Local::now().naive_local().format("%Y-%m-%d/%T").to_string();
        std::io::Write::write(&mut f, now.as_bytes()) .expect("writted");
    }

}

#[allow(unused)]
impl Section {
    pub fn check(&self, profile: &Profile) -> bool {
        let mut modified = self.get_modified(&self.deps_src.keys().cloned().collect(), profile);
        modified.append(&mut self.collect_missing_objects(profile));

        if modified.is_empty() && std::path::Path::new(&format!(".abs/{}/{}/binary/{}", self.name, profile.name, self.name)).exists() {
            println!("{:>RESULT_BORDED_WIDTH$} {}", "Checking".bright_green(), "everything is ok");
            return true;
        }

        let mut is_successful = true;

        let mut built: Vec<String> = vec![];
        let mut compiled_number = 0;

        for modified_file in &modified {
            for for_build in &self.deps_src[modified_file] {
                if built.contains(&for_build.path()) || for_build.path().ends_with(".hpp") || for_build.path().ends_with(".h") {
                    continue;
                }
                let included_directories_argument = self.include_directories.iter().map(|str|format!("-I{}", str));

                let mut child = std::process::Command::new("g++")
                    .args(&profile.options)
                    .arg(&profile.standard)
                    .args(&profile.defines)
                    .args(included_directories_argument)
                    .arg("-fsyntax-only")
                    .arg(&for_build.path())
                    .spawn().unwrap();

                let mut output = String::new();


                    
                match child.wait() {
                    Ok(exit_status) => {
                        let stderr = child.stdout;
                        if stderr.is_some() {
                            let mut reader = std::io::BufReader::new(stderr.unwrap());
                            reader.read_to_string(&mut output);
                        }

                        if exit_status.success() {
                            println!("{:>RESULT_BORDED_WIDTH$} '{}'", "Ok".green().bold(), for_build.path());
                            compiled_number += 1;
                        } else {
                            println!("{:>RESULT_BORDED_WIDTH$} '{}'", "Fail".red().bold(), for_build.path());
                            if !output.is_empty() {
                                println!("{}", output);
                            }
                            is_successful = false;
                            built.push(for_build.path());
                            continue;
                        }
                    },
                    Err(_) => {
                        println!("{:>RESULT_BORDED_WIDTH$} '{}'", "Fail".red().bold(), for_build.path());
                        is_successful = false;
                        built.push(for_build.path());
                    },
                }

                built.push(for_build.path());
            }
        }
        if !is_successful {
            println!("{:>RESULT_BORDED_WIDTH$} Ok {}/{}", "Got errors while checking:".red().bold(), compiled_number, built.len());
            return false;
        }
        println!("{:>RESULT_BORDED_WIDTH$}", "Everything is ok".green().bold());
        return true;
    }

    pub fn link(&self, profile: &Profile) -> bool {
        let objects: Vec<String> = std::fs::read_dir(format!(".abs/{}/{}/binary/", self.name, profile.name)).unwrap()
            .filter(|file|{
            let file = file.as_ref().unwrap();
            return file.file_name().to_str().unwrap().ends_with(".o");
        }).map(|file|{
            let file = file.unwrap();
            return file.path().canonicalize().unwrap().to_str().unwrap().to_string();
        }).collect();
        // linking
        let mut child = std::process::Command::new(&profile.compiler)
            .args(&profile.linking_directories)
            .args(&profile.linking_options)
            .args(objects)
            .arg("-o")
            .arg(format!(".abs/{}/{}/binary/{}", self.name, profile.name, self.name))
            .spawn().unwrap();

        match child.wait() {
            Ok(exit_status) => {
                if exit_status.success() {
                    println!("{:>RESULT_BORDED_WIDTH$} {}", "Complete".green().bold(), "linking".cyan());
                } else {
                    println!("{:>RESULT_BORDED_WIDTH$} {}", "Fail".red().bold(), "linking".cyan());
                    return false;
                }
            },
            Err(_) => {
                println!("{:>RESULT_BORDED_WIDTH$} {}", "Fail".red().bold(), "linking".cyan());
                return false;
            },
        }
        return true;
    }

    pub fn collect_missing_objects(&self, profile: &Profile) -> Vec<File> {
        self.files.iter().filter(|file|{
            if file.path().ends_with(".cpp") || file.path().ends_with(".c") {
                let temp = file.path().replace("/", "|");
                let name = &format!(".abs/{}/{}/binary/{}.o", self.name, profile.name, temp
                    .strip_suffix(".cpp")
                    .or_else(||temp.strip_suffix(".c")).unwrap());
                if !std::path::Path::new(name).exists() {
                    return true;
                }
            }
            return false;
        }).map(|file|file.clone()).collect()
    }

    pub fn check_is_executable_exist(&self, profile: &Profile) -> bool {
        std::path::Path::new(&format!(".abs/{}/{}/binary/{}", self.name, profile.name, self.name)).exists()
    }

    pub fn build(&self, profile: &Profile) -> bool {
        std::fs::create_dir_all(format!(".abs/{}/{}/binary/", self.name, profile.name));

        let mut modified = self.get_modified(&self.deps_src.keys().cloned().collect(), profile);
        modified.append(&mut self.collect_missing_objects(profile));

        if modified.is_empty() && self.check_is_executable_exist(profile) {
            println!("{:>RESULT_BORDED_WIDTH$} {}", "Compiling".bright_green(), "nothing to compile");
            return true;
        }

        let mut built: Vec<&File> = vec![];

        let mut failed: Vec<File> = vec![];

        let mut childs: HashMap<&File, Child> = HashMap::new();

        let mut handle_child = |child: &mut Child, file: &File| -> bool {
            let file = file.clone();
            match child.try_wait() {
                Ok(result) => {
                    if result.is_none() {
                        return true;
                    }
                    let result = result.unwrap();
                    if result.success() {
                        println!("{:>RESULT_BORDED_WIDTH$} '{}'", "Complete".green().bold(), file.path());
                        self.freeze(&file, profile);
                    } else {
                        println!("{:>RESULT_BORDED_WIDTH$} '{}'", "Fail".red().bold(), file.path());
                        failed.push(file);
                    }
                    return false;
                },
                Err(_) => return true
            }
        };

        for modified_file in &modified {
            let mut is_all_compiled = true;
            self.deps_src[modified_file].iter().for_each(|for_build|{
                if built.contains(&for_build) || for_build.path().ends_with(".hpp") || for_build.path().ends_with(".h") {
                    return;
                }
                if for_build.path() != modified_file.path() && match self.get_frozen_time(&for_build, profile) { // if file was frozen after changing dependency
                        Some(for_build_frozen_time) => !modified_file.is_modified(&for_build_frozen_time),
                        None => false,
                    } {
                    return
                }
                let included_directories_argument = self.include_directories.iter().map(|str|format!("-I{}", str));

                while childs.len() >= 8 {
                    childs.retain(|file, child|{
                        handle_child(child, *file)
                    });
                }
                childs.insert(&for_build, std::process::Command::new(&profile.compiler)
                    .arg("-c")
                    .arg(&for_build.path())
                    .args(&profile.options)
                    .arg(&profile.standard)
                    .args(&profile.defines)
                    .args(included_directories_argument)
                    .arg("-o")
                    .arg(&for_build.get_object_path(&self.name, profile))
                    .spawn().unwrap());

                built.push(&for_build);
            });
        }
        while childs.len() > 0 {
            childs.retain(|file, child|{
                handle_child(child, *file)
            });
        }

        for (dep, srcs) in &self.deps_src {
            if srcs.iter().all(|src|{
                !failed.contains(&src)
            }) {
                self.freeze(dep, profile);
            }
        }

        if !failed.is_empty() {
            println!("{:>RESULT_BORDED_WIDTH$} {}. Compiled {}/{}", "Fail".red().bold(), "compiling".cyan(), built.len() - failed.len(), built.len());
            return false;
        }
        println!("{:>RESULT_BORDED_WIDTH$} {}", "Complete".green().bold(), "compiling".cyan());

        return self.link(profile);
    }

    pub fn run(&self, profile: &Profile) -> bool{
        if !self.build(profile) {
            return false;
        }
        println!("{:>RESULT_BORDED_WIDTH$} '{}' with profile '{}'", "Running".bright_green(), self.name, profile.name);
        let mut child = std::process::Command::new(format!(".abs/{}/{}/binary/{}", self.name, profile.name, self.name))
            .spawn().unwrap();
        return true;
    }
}