use std::{vec, collections::HashMap, fs::DirEntry, io::Write};

use chrono::DateTime;
type DateTimeT = DateTime<chrono::offset::Utc>;

fn collect_files<const N: usize>(path: &str, suffixes: [&str; N]) -> Vec<DirEntry> {
    let mut vec_of_pathes = vec![];
    for entry in std::fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let meta = entry.metadata().unwrap();
        let str_name = entry.file_name().to_str().unwrap().to_owned();
        if meta.is_dir() {
            vec_of_pathes.append(&mut collect_files(entry.path().to_str().unwrap(), suffixes));
        } else if suffixes.iter().any(|&suffix|str_name.ends_with(suffix)) {
            vec_of_pathes.push(entry);
        }
    }
    return vec_of_pathes;
}

fn collect_default_includes() -> Vec<String> {
    let includes = std::process::Command::new("sh")
        .arg("-c")
        .arg("c++ -xc++ /dev/null -E -Wp,-v 2>&1 | sed -n 's,^ ,,p'").output().expect("failed to execute process");

    let includes = String::from_utf8_lossy(&includes.stdout);
    let includes: Vec<String> = includes.split("\n").map(str::to_string).collect();
    includes
}

fn is_dependency_exist(dependency: &str, directory: &str) -> bool {
    return std::path::Path::new(&format!("{}/{}", directory, dependency)).exists();
}

fn get_dependency_path(dependency: &str, directories: &[String]) -> Option<String> {
    for dir in directories {
        if is_dependency_exist(dependency, dir) {
            return Some(format!("{}/{}", dir, dependency));
        }
    }
    return None;
}

fn collect_local_dependencies_for_file(path: &str, search_list: &[String]) -> Vec<String> {
    let temp = std::path::Path::new(path).canonicalize().unwrap();
    let path_to_file = temp.parent().unwrap().to_str().unwrap();

    let file = std::fs::File::open(path).unwrap();
    let reader = std::io::BufReader::new(file);

    let mut dependencies: Vec<String> = vec![];
    for line in std::io::BufRead::lines(reader) {
        let line = line.unwrap();
        if line.starts_with("#include") {
            let stripped = line.strip_prefix("#include ").unwrap().trim();
            let name: String = stripped[1..stripped.len() - 1].to_string();
            // check local
            if is_dependency_exist(&name, path_to_file) {
                dependencies.push(format!("{}/{}", path_to_file, name));
                continue;
            }
            // check specialized pathes
            let path = get_dependency_path(&name, search_list)
                .expect(&(String::from("Dependency doesn't exist: ") + &name));
            dependencies.push(path);
        }
    }
    return dependencies;
}

fn create_map_source_dependencies(pathes: &Vec<DirEntry>, search_list: &Vec<String>) -> HashMap<String, Vec<String>> {
    let mut map = HashMap::new();

    for path in pathes {
        let path = path.path().canonicalize().unwrap().to_str().unwrap().to_owned();
        map.insert(path.to_owned(), collect_local_dependencies_for_file(&path, &search_list));
    }
    return map;
}

fn system_time_to_datetime(system_time: std::time::SystemTime) -> DateTimeT {
    system_time.into()
}

fn datetime_to_string(date_time: DateTimeT) -> String {
    date_time.format("%Y-%m-%d/%T/%z").to_string()
}

fn create_map_dependecy_sources(pathes: &Vec<DirEntry>, search_list: &Vec<String>) -> HashMap<String, Vec<String>> {
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
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

fn get_modified(files: &Vec<DirEntry>) -> Vec<String> {
    let mut modified: Vec<String> = vec![];

    let f = std::fs::File::open(".abs_frozen").expect("Unable to open file");
    let f = std::io::BufReader::new(f);
    let mut frozen: HashMap<String, DateTimeT> = HashMap::new();
    std::io::BufRead::lines(f).for_each(|s| {
        let line = s.unwrap();
        let strings: Vec<&str> = line.split_whitespace().collect();
        let time: DateTimeT = DateTime::parse_from_str(strings[1], "%Y-%m-%d/%T/%z").unwrap().into();
        frozen.insert(String::from(strings[0]), time);
    });

    for file in files {
        let p = file.path();
        let path = p.to_str().unwrap();
        if frozen.contains_key(path) {
            if frozen[path].timestamp() != system_time_to_datetime(file.metadata().unwrap().modified().unwrap()).timestamp() {
                modified.push(path.to_owned());
            }
        } else {
            modified.push(path.to_owned());
        }
    }
    modified
}

fn freeze(files: &Vec<DirEntry>) {
    let f = std::fs::File::create(".abs_frozen").expect("Unable to create file");
    let mut f = std::io::BufWriter::new(f);
    for file in files {
        let time = system_time_to_datetime(file.metadata().unwrap().modified().unwrap());
        let row = format!("{} {}\n", file.path().to_str().unwrap(), datetime_to_string(time));
        f.write(row.as_bytes()).expect("writted");
    }
}

fn main() {
    let all_files = collect_files("test_data/", [".hpp", ".cpp", ".h", ".c"]);
    let modified = get_modified(&all_files);
    println!("Modified: {:?}", modified);
    freeze(&all_files);
    println!("All files: {:#?}", all_files);

    let default_includes = collect_default_includes();

    let src_dep = create_map_source_dependencies(&all_files, &default_includes);
    println!("Src -> dep: {:#?}", src_dep);
    let dep_src = create_map_dependecy_sources(&all_files, &default_includes);
    println!("Dep -> src: {:#?}", dep_src);
}
