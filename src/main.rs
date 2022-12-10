use std::{vec, collections::HashMap, io::Write};

use chrono::NaiveDateTime;
type DateTimeT = NaiveDateTime;

#[derive(Hash, PartialEq, PartialOrd, Eq, Debug, Clone)]
struct File {
    name: String,
    last_modification: DateTimeT
}

impl File {
    fn new(name: String, last_modification: DateTimeT) -> File {
        File { name, last_modification }
    }
}

fn collect_files<const N: usize>(path: &str, suffixes: [&str; N]) -> Vec<File> {
    let mut vec_of_pathes = vec![];
    for entry in std::fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let meta = entry.metadata().unwrap();
        let abs = entry.path().canonicalize().unwrap();
        let full_path = abs.to_str().unwrap();
        if meta.is_dir() {
            vec_of_pathes.append(&mut collect_files(full_path, suffixes));
        } else if suffixes.iter().any(|&suffix|full_path.ends_with(suffix)) {
            vec_of_pathes.push(File::new(full_path.to_owned(), system_time_to_datetime(meta.modified().unwrap())));
        }
    }
    return vec_of_pathes;
}

fn collect_default_includes() -> Vec<String> {
    let includes = std::process::Command::new("sh")
        .arg("-c")
        .arg("c++ -xc++ /dev/null -E -Wp,-v 2>&1 | sed -n 's,^ ,,p'").output().expect("failed to execute process");

    let includes = String::from_utf8(includes.stdout).unwrap();
    let includes: Vec<String> = includes.split("\n").map(str::to_string).collect();
    includes
}

fn is_dependency_exist(dependency: &str, directory: &str) -> bool {
    return std::path::Path::new(&format!("{}/{}", directory, dependency)).exists();
}

fn get_dependency_path(dependency: &str, directories: &[String]) -> Option<File> {
    for dir in directories {
        if is_dependency_exist(dependency, dir) {
            let dependency_path = format!("{}/{}", dir, dependency);
            let modified = system_time_to_datetime(std::path::Path::new(&dependency_path).metadata().unwrap().modified().unwrap());
            return Some(File::new(dependency_path, modified));
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
        if is_dependency_exist(&name, path_to_file) {
            let path = format!("{}/{}", path_to_file, name);
            let modified = system_time_to_datetime(std::path::Path::new(&path).metadata().unwrap().modified().unwrap());
            return File::new(path, modified);
        }
        // check specialized pathes
        return get_dependency_path(&name, search_list)
            .expect(&(String::from("Dependency doesn't exist: ") + &name));
    }).collect()
}

fn create_map_source_dependencies(pathes: &Vec<File>, search_list: &Vec<String>) -> HashMap<File, Vec<File>> {
    let mut map: HashMap<File, Vec<File>> = HashMap::new();
    for path in pathes {
        map.insert(path.to_owned(), collect_local_dependencies_for_file(&path.name, &search_list).into_iter().chain(vec![path.to_owned()].into_iter()).collect());
    }
    return map;
}

fn system_time_to_datetime(system_time: std::time::SystemTime) -> DateTimeT {
    chrono::DateTime::<chrono::Local>::from(system_time).naive_local()
}

fn datetime_to_string(date_time: DateTimeT) -> String {
    date_time.format("%Y-%m-%d/%T").to_string()
}

fn create_map_dependecy_sources(pathes: &Vec<File>, search_list: &Vec<String>) -> HashMap<File, Vec<File>> {
    let mut map: HashMap<File, Vec<File>> = HashMap::new();
    let src_dep = create_map_source_dependencies(pathes, search_list);
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

fn get_modified(files: &Vec<File>) -> Vec<File> {
    let mut files = files.clone();
    let f = std::fs::File::open(".abs_frozen");
    if f.is_err() {
        std::fs::File::create(".abs_frozen").expect("Create .abs_frozen");
        return files;
    }
    let f = f.unwrap();

    let f = std::io::BufReader::new(f);

    let mut modified: Vec<File> = std::io::BufRead::lines(f).filter_map(|s| {
        let line = s.unwrap();
        let strings: Vec<&str> = line.split_whitespace().collect();
        let path = strings[0];
        let time: DateTimeT = NaiveDateTime::parse_from_str(strings[1], "%Y-%m-%d/%T").unwrap().into();
        let mut changed: Option<File> = None;
        files.retain(|file| {
            if path == file.name {
                if time.timestamp() != file.last_modification.timestamp() {
                    changed = Some(file.to_owned());
                }
                return false;
            } else {
                return true;
            }
        });
        if changed.is_some() {
            changed
        } else {
            None
        }
    }).collect();
    modified.append(&mut files);
    return modified;
}

fn freeze(files: &Vec<File>) {
    let f = std::fs::File::create(".abs_frozen").expect("Unable to create file");
    let mut f = std::io::BufWriter::new(f);
    for file in files {
        let row = format!("{} {}\n", file.name, datetime_to_string(file.last_modification));
        f.write(row.as_bytes()).expect("writted");
    }
}

fn main() {
    let all_files = collect_files("test_data/", [".hpp", ".cpp", ".h", ".c"]);

    let default_includes = collect_default_includes();

    let src_dep = create_map_source_dependencies(&all_files, &default_includes);
    println!("Src -> dep: {:#?}", src_dep);
    let dep_src = create_map_dependecy_sources(&all_files, &default_includes);
    println!("Dep -> src: {:#?}", dep_src);

    let dependencies = &dep_src.into_keys().collect();
    let modified = get_modified(&dependencies);
    println!("Modified: {:#?}", modified);
    freeze(&dependencies);
    println!("All files: {:#?}", all_files);
}
