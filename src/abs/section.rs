use super::{file::File, profile::Profile};
use chrono::NaiveDateTime;
use colored::Colorize;
use std::{collections::HashMap, io::Read, process::Child, rc::Weak, cell::RefCell};
use super::tank::Tank;

#[derive(Debug)]
pub enum SectionError {
    MandatoryLack(String),
    FieldTypeError(String),
}

#[allow(unused)]
#[derive(Debug)]
pub struct Section {
    pub name: String,
    pub output_type: String,
    pipes: Vec<Weak<RefCell<Section>>>,
    files: Vec<File>,
    include_directories: Vec<String>,
    deps_src: HashMap<File, Vec<File>>,
    srcs_dep: HashMap<File, Vec<File>>,
}

const RESULT_BORDER_WIDTH: usize = 10;

#[allow(unused)]
impl Section {
    pub fn new(tank: &Tank, name: String, config: &toml::Value) -> Result<Section, SectionError> {
        std::fs::create_dir_all(format!(".abs/{}", name));

        let mut section_files: Vec<File> = vec![];

        let source_dir = config
            .get("source")
            .ok_or(SectionError::MandatoryLack(
                "'source' is mandatory field!".to_string(),
            ))?
            .as_str()
            .ok_or(SectionError::FieldTypeError(
                "'source' is string type!".to_string(),
            ))?;

        section_files = Section::collect_files(source_dir, [".hpp", ".cpp", ".h", ".c"]);

        let include_dir = config.get("include");

        let output_type = match config.get("type") {
            Some(value) => {
                if value.is_str() {
                    value.as_str().unwrap().to_string()
                } else {
                    println!(
                        "{:>RESULT_BORDER_WIDTH$}",
                        "Expected string for 'type'".red().bold()
                    );
                    std::process::exit(1);
                }
            },
            None => "executable".to_string(),
        };

        let mut include_directories = Section::collect_default_includes();
        include_directories.push(source_dir.to_string());

        if include_dir.is_some() {
            let include_dir = include_dir
                .unwrap()
                .as_str()
                .ok_or(SectionError::FieldTypeError(
                    "'source' is string type!".to_string(),
                ))?;
            section_files.append(&mut Section::collect_files(
                include_dir,
                [".hpp", ".cpp", ".h", ".c"],
            ));

            include_directories.push(include_dir.to_string());
        }

        let deps_src = Section::create_map_dependency_sources(&section_files, &include_directories);
        let srcs_dep =
            Section::create_map_source_dependencies(&section_files, &include_directories);

        let pipes = match config.get("pipes") {
            Some(value) =>  {
                if value.is_array() {
                    return Err(SectionError::FieldTypeError("pipes must be an array".to_string()));
                }
                value.as_array().unwrap()
                    .iter()
                    .map(|elem| {
                        if elem.is_str() {
                            let name = elem.as_str().unwrap().split(".").collect::<Vec<&str>>()[1];

                            return Weak::clone(tank.get_sections()
                                .iter()
                                .find(|section| {
                                    let section = match section.upgrade() {
                                        Some(ptr) => ptr,
                                        None => todo!(),
                                    };
                                    let section = section.borrow_mut();

                                    return section.name == name;
                            }).unwrap());
                        } else {
                            println!(
                                "{:>RESULT_BORDER_WIDTH$}",
                                " Wrong type for pipe".bright_red()
                            );
                            std::process::exit(1);
                        }})
                    .collect()
            },
            None => vec![],
        };

        Ok(Section {
            name,
            output_type,
            pipes,
            files: section_files,
            include_directories,
            deps_src,
            srcs_dep,
        })
    }

    fn collect_files<const N: usize>(path: &str, suffixes: [&str; N]) -> Vec<File> {
        let mut vec_of_paths = vec![];
        for entry in std::fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let meta = entry.metadata().unwrap();
            let abs = entry.path().canonicalize().unwrap();
            let full_path = abs.to_str().unwrap();
            if meta.is_dir() {
                vec_of_paths.append(&mut Section::collect_files(full_path, suffixes));
            } else if suffixes.iter().any(|&suffix| full_path.ends_with(suffix)) {
                vec_of_paths.push(File::from_path(full_path.to_owned()).unwrap());
            }
        }
        return vec_of_paths;
    }

    fn collect_default_includes() -> Vec<String> {
        let includes = std::process::Command::new("sh")
            .arg("-c")
            .arg("c++ -xc++ /dev/null -E -Wp,-v 2>&1 | sed -n 's,^ ,,p'")
            .output()
            .expect("failed to execute process");

        let includes = String::from_utf8(includes.stdout).unwrap();
        let includes: Vec<String> = includes
            .split("\n")
            .map(str::to_string)
            .filter(|str| !str.is_empty())
            .collect();
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

        std::io::BufRead::lines(reader)
            .map(|line| line.unwrap())
            .filter(|line| line.starts_with("#include"))
            .map(|line| {
                let stripped = line.strip_prefix("#include ").unwrap().trim();
                let name: String = stripped[1..stripped.len() - 1].to_string();
                // check local
                if Section::is_dependency_exist(&name, path_to_file) {
                    let path = format!("{}/{}", path_to_file, name);
                    return File::from_path(path).unwrap();
                }
                // check specialized paths
                if let Some(val) = Section::get_dependency_path(&name, search_list) {
                    return val;
                } else {
                    println!(
                        "{:>RESULT_BORDER_WIDTH$} {}",
                        "Failed to find ".bright_red(),
                        name
                    );
                    std::process::exit(1);
                }
            })
            .collect()
    }

    fn create_map_source_dependencies(
        paths: &Vec<File>,
        search_list: &Vec<String>,
    ) -> HashMap<File, Vec<File>> {
        let mut map: HashMap<File, Vec<File>> = HashMap::new();
        for path in paths {
            map.insert(
                path.to_owned(),
                Section::collect_local_dependencies_for_file(&path.path(), &search_list)
                    .into_iter()
                    .chain(vec![path.to_owned()].into_iter())
                    .collect(),
            );
        }
        return map;
    }

    fn create_map_dependency_sources(
        paths: &Vec<File>,
        search_list: &Vec<String>,
    ) -> HashMap<File, Vec<File>> {
        let mut map: HashMap<File, Vec<File>> = HashMap::new();
        let src_dep = Section::create_map_source_dependencies(paths, search_list);
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
        let f = std::fs::File::open(file.get_freeze_path(&self.name, profile));
        if f.is_err() {
            return None;
        }
        let mut reader = std::io::BufReader::new(f.unwrap());
        let mut content = String::new();
        let res = std::io::BufRead::read_line(&mut reader, &mut content)
            .expect("expect string in the file");

        let time = NaiveDateTime::parse_from_str(&content, "%Y-%m-%d/%T");
        if time.is_err() {
            return None;
        }
        Some(time.unwrap())
    }

    fn get_modified(&self, files: &Vec<File>, profile: &Profile) -> Vec<File> {
        let mut files = files.clone();
        files.retain(|file| match self.get_frozen_time(file, profile) {
            Some(frozen_time) => file.is_modified(&frozen_time),
            None => true,
        });
        return files;
    }

    fn freeze(&self, file: &File, profile: &Profile) {
        std::fs::create_dir_all(format!(".abs/{}/{}/frozen/", self.name, profile.name));
        let f = std::fs::File::create(file.get_freeze_path(&self.name, profile))
            .expect("Unable to create file");
        let mut f = std::io::BufWriter::new(f);
        let file_changed = file.modified().format("%Y-%m-%d/%T").to_string();
        let now = chrono::Local::now()
            .naive_local()
            .format("%Y-%m-%d/%T")
            .to_string();
        std::io::Write::write(&mut f, now.as_bytes()).expect("wrote");
    }
}

#[allow(unused)]
impl Section {
    pub fn check(&self, profile: &Profile) -> bool {
        let mut modified = self.get_modified(&self.deps_src.keys().cloned().collect(), profile);
        modified.append(&mut self.collect_missing_objects(profile));

        if modified.is_empty()
            && std::path::Path::new(&format!(
                ".abs/{}/{}/binary/{}",
                self.name, profile.name, self.name
            ))
            .exists()
        {
            println!(
                "{:>RESULT_BORDER_WIDTH$} {}",
                "Checking".bright_green(),
                "everything is ok"
            );
            return true;
        }

        let mut is_successful = true;

        let mut built: Vec<String> = vec![];
        let mut compiled_number = 0;

        for modified_file in &modified {
            for for_build in &self.deps_src[modified_file] {
                if built.contains(&for_build.path())
                    || for_build.path().ends_with(".hpp")
                    || for_build.path().ends_with(".h")
                {
                    continue;
                }
                let included_directories_argument = self
                    .include_directories
                    .iter()
                    .map(|str| format!("-I{}", str));

                let mut child = std::process::Command::new("g++")
                    .args(&profile.options)
                    .arg(&profile.standard)
                    .args(&profile.defines)
                    .args(included_directories_argument)
                    .arg("-fsyntax-only")
                    .arg(&for_build.path())
                    .spawn()
                    .unwrap();

                let mut output = String::new();

                match child.wait() {
                    Ok(exit_status) => {
                        let stderr = child.stdout;
                        if stderr.is_some() {
                            let mut reader = std::io::BufReader::new(stderr.unwrap());
                            reader.read_to_string(&mut output);
                        }

                        if exit_status.success() {
                            println!(
                                "{:>RESULT_BORDER_WIDTH$} '{}'",
                                "Ok".green().bold(),
                                for_build.path()
                            );
                            compiled_number += 1;
                        } else {
                            println!(
                                "{:>RESULT_BORDER_WIDTH$} '{}'",
                                "Fail".red().bold(),
                                for_build.path()
                            );
                            if !output.is_empty() {
                                println!("{}", output);
                            }
                            is_successful = false;
                            built.push(for_build.path());
                            continue;
                        }
                    }
                    Err(_) => {
                        println!(
                            "{:>RESULT_BORDER_WIDTH$} '{}'",
                            "Fail".red().bold(),
                            for_build.path()
                        );
                        is_successful = false;
                        built.push(for_build.path());
                    }
                }

                built.push(for_build.path());
            }
        }
        if !is_successful {
            println!(
                "{:>RESULT_BORDER_WIDTH$} Ok {}/{}",
                "Got errors while checking:".red().bold(),
                compiled_number,
                built.len()
            );
            return false;
        }
        println!(
            "{:>RESULT_BORDER_WIDTH$}",
            "Everything is ok".green().bold()
        );
        return true;
    }

    pub fn link(&self, profile: &Profile) -> bool {
        let objects: Vec<String> =
            // todo mb just collect files from prev state?
            std::fs::read_dir(format!(".abs/{}/{}/binary/", self.name, profile.name))
                .unwrap()
                .filter(|file| {
                    let file = file.as_ref().unwrap();
                    return file.file_name().to_str().unwrap().ends_with(".o");
                })
                .map(|file| {
                    let file = file.unwrap();
                    return file
                        .path()
                        .canonicalize()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string();
                })
                .collect();
        // linking
        if (self.output_type == "executable") {
            let mut child = std::process::Command::new(&profile.compiler)
                .args(&profile.linking_directories)
                .args(&profile.linking_options)
                .args(objects)
                .arg("-o")
                .arg(format!(
                    ".abs/{}/{}/{}",
                    self.name, profile.name, self.name
                ))
                .spawn()
                .unwrap();

            match child.wait() {
                Ok(exit_status) => {
                    if exit_status.success() {
                        println!(
                            "{:>RESULT_BORDER_WIDTH$} {}",
                            "Complete executable".green().bold(),
                            "linking".cyan()
                        );
                    } else {
                        println!(
                            "{:>RESULT_BORDER_WIDTH$} {}",
                            "Fail".red().bold(),
                            "linking".cyan()
                        );
                        return false;
                    }
                }
                Err(_) => {
                    println!(
                        "{:>RESULT_BORDER_WIDTH$} {}",
                        "Fail".red().bold(),
                        "linking".cyan()
                    );
                    return false;
                }
            }
        } else if (self.output_type == "library") {
            let mut child = std::process::Command::new("ar")
                .arg("rcs")
                .arg("-o")
                .arg(format!(
                    ".abs/{}/{}/lib{}.a",
                    self.name, profile.name, self.name
                ))
                .args(objects)
                .spawn()
                .unwrap();

            match child.wait() {
                Ok(exit_status) => {
                    if exit_status.success() {
                        println!(
                            "{:>RESULT_BORDER_WIDTH$} {}",
                            "Complete static library".green().bold(),
                            "linking".cyan()
                        );
                    } else {
                        println!(
                            "{:>RESULT_BORDER_WIDTH$} {}",
                            "Fail".red().bold(),
                            "linking".cyan()
                        );
                        return false;
                    }
                }
                Err(_) => {
                    println!(
                        "{:>RESULT_BORDER_WIDTH$} {}",
                        "Fail".red().bold(),
                        "linking".cyan()
                    );
                    return false;
                }
            }
        }
        return true;
    }

    pub fn collect_missing_objects(&self, profile: &Profile) -> Vec<File> {
        self.files
            .iter()
            .filter(|file| {
                if file.path().ends_with(".cpp") || file.path().ends_with(".c") {
                    let name = &file.get_object_path(&self.name, profile);
                    if !std::path::Path::new(&name).exists() {
                        return true;
                    }
                }
                return false;
            })
            .map(|file| file.clone())
            .collect()
    }

    pub fn check_is_executable_exist(&self, profile: &Profile) -> bool {
        std::path::Path::new(&format!(
            ".abs/{}/{}/binary/{}",
            self.name, profile.name, self.name
        ))
        .exists()
    }

    pub fn build(&self, profile: &Profile) -> bool {
        std::fs::create_dir_all(format!(".abs/{}/{}/binary/", self.name, profile.name));

        let mut modified = self.get_modified(&self.deps_src.keys().cloned().collect(), profile);

        modified.append(&mut self.collect_missing_objects(profile));

        if modified.is_empty() && self.check_is_executable_exist(profile) {
            println!(
                "{:>RESULT_BORDER_WIDTH$} {}",
                "Compiling".bright_green(),
                "nothing to compile"
            );
            return true;
        }

        let mut built: Vec<&File> = vec![];

        let mut failed: Vec<File> = vec![];

        let mut children: HashMap<&File, Child> = HashMap::new();

        let mut handle_child = |child: &mut Child, file: &File| -> bool {
            let file = file.clone();
            match child.try_wait() {
                Ok(result) => {
                    if result.is_none() {
                        return true;
                    }
                    let result = result.unwrap();
                    if result.success() {
                        println!(
                            "{:>RESULT_BORDER_WIDTH$} '{}'",
                            "Complete".green().bold(),
                            file.path()
                        );
                        self.freeze(&file, profile);
                    } else {
                        println!(
                            "{:>RESULT_BORDER_WIDTH$} '{}'",
                            "Fail".red().bold(),
                            file.path()
                        );
                        failed.push(file);
                    }
                    return false;
                }
                Err(_) => return true,
            }
        };

        for modified_file in &modified {
            let mut is_all_compiled = true;
            self.deps_src[modified_file].iter().for_each(|for_build| {
                if built.contains(&for_build)
                    || for_build.path().ends_with(".hpp")
                    || for_build.path().ends_with(".h")
                {
                    return;
                }
                if for_build.path() != modified_file.path()
                    && match self.get_frozen_time(&for_build, profile) {
                        // if file was frozen after changing dependency
                        Some(for_build_frozen_time) => {
                            !modified_file.is_modified(&for_build_frozen_time)
                        }
                        None => false,
                    }
                {
                    return;
                }
                let included_directories_argument = self
                    .include_directories
                    .iter()
                    .map(|str| format!("-I{}", str));

                while children.len() >= 8 {
                    children.retain(|file, child| handle_child(child, *file));
                }
                children.insert(
                    &for_build,
                    std::process::Command::new(&profile.compiler)
                        .arg("-c")
                        .arg(&for_build.path())
                        .args(&profile.options)
                        .arg(&profile.standard)
                        .args(&profile.defines)
                        .args(included_directories_argument)
                        .arg("-o")
                        .arg(&for_build.get_object_path(&self.name, profile))
                        .spawn()
                        .unwrap(),
                );

                built.push(&for_build);
            });
        }
        while children.len() > 0 {
            children.retain(|file, child| handle_child(child, *file));
        }

        for (dep, srcs) in &self.deps_src {
            if srcs.iter().all(|src| !failed.contains(&src)) {
                self.freeze(dep, profile);
            }
        }

        if !failed.is_empty() {
            println!(
                "{:>RESULT_BORDER_WIDTH$} {}. Compiled {}/{}",
                "Fail".red().bold(),
                "compiling".cyan(),
                built.len() - failed.len(),
                built.len()
            );
            return false;
        }
        println!(
            "{:>RESULT_BORDER_WIDTH$} {}",
            "Complete".green().bold(),
            "compiling".cyan()
        );

        return self.link(profile);
    }

    pub fn run(&self, profile: &Profile) -> bool {
        if !self.build(profile) {
            return false;
        }
        println!(
            "{:>RESULT_BORDER_WIDTH$} '{}' with profile '{}'",
            "Running".bright_green(),
            self.name,
            profile.name
        );
        let mut child = std::process::Command::new(format!(
            ".abs/{}/{}/{}",
            self.name, profile.name, self.name
        ))
        .spawn()
        .unwrap();
        return true;
    }

    pub fn get_output(&self, profile: &Profile) -> String {
        if self.output_type == "executable" {
            format!(
                ".abs/{}/{}/{}",
                self.name, profile.name, self.name)
        } else {
            format!(
                ".abs/{}/{}/lib{}.a",
                self.name, profile.name, self.name)
        }
    }
}
