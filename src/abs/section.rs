use super::{file::File, profile::Profile};
use chrono::NaiveDateTime;
use colored::Colorize;
use std::{collections::HashMap, io::Read, process::Child, rc::Rc, rc::Weak, cell::RefCell, path::Path, fmt::format};
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
    pub outlet_type: String,
    profile: Rc<Profile>,
    pipes: Vec<Weak<RefCell<Section>>>,
    files: Vec<File>,
    include_directories: Vec<String>,
    sources_of_dependency: HashMap<File, Vec<File>>,
    dependencies_of_source: HashMap<File, Vec<File>>,
}

pub const RESULT_BORDER_WIDTH: usize = 10;

#[allow(unused)]
impl Section {
    pub fn new(tank: &Tank, name: String, config: &toml::Value, profile: Rc<Profile>) -> Result<Section, SectionError> {
        std::fs::create_dir_all(format!(".abs/{}", name));

        let source_dir = config
            .get("source")
            .ok_or(SectionError::MandatoryLack(
                "'source' is mandatory field!".to_string(),
            ))?
            .as_str()
            .ok_or(SectionError::FieldTypeError(
                "'source' is string type!".to_string(),
            ))?;

        let mut section_files = File::collect_files(source_dir, [".hpp", ".cpp", ".h", ".c"]);

        let mut include_directories = Section::collect_default_includes();
        include_directories.push(source_dir.to_string());

        if let Some(include_dir) = config.get("include") {
            let include_dir = include_dir
                .as_str()
                .ok_or(SectionError::FieldTypeError(
                    "'source' is string type!".to_string(),
                ))?;
            section_files.append(&mut File::collect_files(
                include_dir,
                [".hpp", ".cpp", ".h", ".c"],
            ));
            include_directories.push(include_dir.to_string());
        }

        let mut outlet_type = String::from("executable");
        if let Some(value) = config.get("type") {
            if value.is_str() {
                outlet_type = value.as_str().unwrap().to_string(); // todo check correct type
            } else {
                println!(
                    "{:>RESULT_BORDER_WIDTH$}",
                    "Expected string for 'type'".red().bold()
                );
                std::process::exit(1);
            }
        }

        let sources_of_dependency = 
            Section::create_map_dependency_sources(&section_files, &include_directories);
        let dependencies_of_source =
            Section::create_map_source_dependencies(&section_files, &include_directories);
    
        let mut pipes = vec![];
        if let Some(value) = config.get("pipes") {
            if let Some(value ) = value.as_array()
            {
                pipes = value
                    .iter()
                    .map(|elem| {
                        if elem.is_str() {
                            let name = elem.as_str().unwrap().split(".").collect::<Vec<&str>>()[1];

                            return Weak::clone(tank.get_sections()
                                .iter()
                                .find(|section| {
                                    if let Some(section) = section.upgrade() {
                                        let section = section.borrow_mut();
                                        return section.name == name;
                                    } else {
                                        return false;
                                    }
                                })
                                .unwrap()); // todo doesn't exist case
                        } else {
                            println!(
                            "{:>RESULT_BORDER_WIDTH$}",
                            " Wrong type for pipe".bright_red()
                        );
                        std::process::exit(1);
                    }})
                    .collect()
            } else {
                return Err(SectionError::FieldTypeError("pipes must be an array".to_string()));
            }
        }

        Ok(Section {
            name,
            outlet_type,
            profile,
            pipes,
            files: section_files,
            include_directories,
            sources_of_dependency,
            dependencies_of_source,
        })
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

    fn create_map_source_dependencies(
        paths: &Vec<File>,
        search_list: &Vec<String>,
    ) -> HashMap<File, Vec<File>> {
        let mut map: HashMap<File, Vec<File>> = HashMap::new();
        for path in paths {
            map.insert(
                path.to_owned(),
                path.collect_dependencies(search_list)
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

    fn get_modified(&self, files: &Vec<File>) -> Vec<File> {
        let mut files = files.clone();
        files.retain(|file| file.is_modified(&self.name, &self.profile));
        return files;
    }

    fn freeze(&self, file: &File) {
        std::fs::create_dir_all(self.get_frozen_path());
        let f = std::fs::File::create(file.get_freeze_path(&self.name, &self.profile))
            .expect("Unable to create file");
        let mut f = std::io::BufWriter::new(f);
        let file_changed = file.modified().format("%Y-%m-%d/%T").to_string();
        let now = chrono::Local::now()
            .naive_local()
            .format("%Y-%m-%d/%T")
            .to_string();
        std::io::Write::write(&mut f, now.as_bytes()).expect("wrote");
    }

    pub fn check_is_outlet_exist(&self) -> bool {
        Path::new(&self.get_outlet_path()).exists()
    }

    pub fn get_outlet_path(&self) -> String {
        if self.outlet_type == "executable"{
            format!(
                ".abs/{}/{}/{}",
                self.name, self.profile.name, self.name)
        } else if self.outlet_type == "library" {
            format!(
                ".abs/{}/{}/lib{}.a",
                self.name, self.profile.name, self.name
            )
        } else if self.outlet_type == "shared" {
            format!(
                ".abs/{}/{}/lib{}.so",
                self.name, self.profile.name, self.name
            )
        } else {
            panic!("unexpected outlet type {}", self.outlet_type);
        }
    }

    pub fn get_binary_path(&self) -> String {
        format!(".abs/{}/{}/binary/", self.name, self.profile.name)
    }
    pub fn get_frozen_path(&self) -> String {
        format!(".abs/{}/{}/frozen/", self.name, self.profile.name)
    }
}

#[allow(unused)]
impl Section {
    pub fn check(&self) -> bool {
        let mut modified = self.get_modified(&self.sources_of_dependency.keys().cloned().collect());
        modified.append(&mut self.collect_missing_objects());

        if modified.is_empty() && self.check_is_outlet_exist() {
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
            for for_build in &self.sources_of_dependency[modified_file] {
                if built.contains(&for_build.path)
                    || for_build.path.ends_with(".hpp")
                    || for_build.path.ends_with(".h")
                {
                    continue;
                }
                let included_directories_argument = self
                    .include_directories
                    .iter()
                    .map(|str| format!("-I{}", str));

                let mut child = std::process::Command::new("g++")
                    .args(&self.profile.options)
                    .arg(&self.profile.standard)
                    .args(&self.profile.defines)
                    .args(included_directories_argument)
                    .arg("-fsyntax-only")
                    .arg(&for_build.path)
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
                                for_build.path
                            );
                            compiled_number += 1;
                        } else {
                            println!(
                                "{:>RESULT_BORDER_WIDTH$} '{}'",
                                "Fail".red().bold(),
                                for_build.path
                            );
                            if !output.is_empty() {
                                println!("{}", output);
                            }
                            is_successful = false;
                            built.push(for_build.path.clone());
                            continue;
                        }
                    }
                    Err(_) => {
                        println!(
                            "{:>RESULT_BORDER_WIDTH$} '{}'",
                            "Fail".red().bold(),
                            for_build.path
                        );
                        is_successful = false;
                        built.push(for_build.path.clone());
                    }
                }

                built.push(for_build.path.clone());
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

    pub fn link(&self) -> bool {
        let objects: Vec<String> =
            // todo mb just collect files from prev state?
            std::fs::read_dir(self.get_binary_path())
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
        if (self.outlet_type == "executable") {
            let mut child = std::process::Command::new(&self.profile.compiler)
                .args(&self.profile.linking_directories)
                .args(&self.profile.linking_options)
                .args(objects)
                .arg("-o")
                .arg(format!(
                    ".abs/{}/{}/{}",
                    self.name, self.profile.name, self.name
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
        } else if (self.outlet_type == "library") {
            let mut child = std::process::Command::new("ar")
                .arg("rcs")
                .arg("-o")
                .arg(format!(
                    ".abs/{}/{}/lib{}.a",
                    self.name, self.profile.name, self.name
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

    pub fn collect_missing_objects(&self) -> Vec<File> {
        self.files
            .iter()
            .filter(|file| {
                if file.path.ends_with(".cpp") || file.path.ends_with(".c") {
                    let name = &file.get_object_path(&self.name, &self.profile);
                    if !Path::new(&name).exists() {
                        return true;
                    }
                }
                return false;
            })
            .map(|file| file.clone())
            .collect()
    }

    pub fn build(&self) -> bool {
        std::fs::create_dir_all(self.get_binary_path());

        let mut modified = self.get_modified(&self.sources_of_dependency.keys().cloned().collect());

        modified.append(&mut self.collect_missing_objects());

        if modified.is_empty() && self.check_is_outlet_exist() {
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
                            file.path
                        );
                        self.freeze(&file);
                    } else {
                        println!(
                            "{:>RESULT_BORDER_WIDTH$} '{}'",
                            "Fail".red().bold(),
                            file.path
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
            self.sources_of_dependency[modified_file].iter().for_each(|for_build| {
                if built.contains(&for_build)
                    || for_build.path.ends_with(".hpp")
                    || for_build.path.ends_with(".h") {
                    return;
                }
                if for_build.path != modified_file.path
                    && for_build.is_modified(&self.name, &self.profile) {
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
                    std::process::Command::new(&self.profile.compiler)
                        .arg("-c")
                        .arg(&for_build.path)
                        .args(&self.profile.options)
                        .arg(&self.profile.standard)
                        .args(&self.profile.defines)
                        .args(included_directories_argument)
                        .arg("-o")
                        .arg(&for_build.get_object_path(&self.name, &self.profile))
                        .spawn()
                        .unwrap(),
                );

                built.push(&for_build);
            });
        }
        while children.len() > 0 {
            children.retain(|file, child| handle_child(child, *file));
        }

        for (dep, srcs) in &self.sources_of_dependency {
            if srcs.iter().all(|src| !failed.contains(&src)) {
                self.freeze(dep);
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

        return self.link();
    }

    pub fn run(&self) -> bool {
        if self.outlet_type != "executable" {
            println!(
                "{:>RESULT_BORDER_WIDTH$}",
                "Can't run non executable outlet"
            );
            std::process::exit(1);
        }
        if !self.build() {
            return false;
        }
        println!(
            "{:>RESULT_BORDER_WIDTH$} '{}' with profile '{}'",
            "Running".bright_green(),
            self.name,
            self.profile.name
        );
        let mut child = std::process::Command::new(format!(
            ".abs/{}/{}/{}",
            self.name, self.profile.name, self.name
        ))
        .spawn()
        .unwrap();
        return true;
    }
}
