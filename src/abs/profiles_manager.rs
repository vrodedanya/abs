use std::collections::HashMap;
use std::rc::Rc;
use super::profile::Profile;

#[derive(Debug)]
#[derive(Clone)]
#[allow(unused)]
pub struct ProfilesManager {
    profiles: HashMap<String, Rc<Profile>>,
}

#[allow(unused)]
impl ProfilesManager {
    pub fn new(config: Option<&toml::Value>) -> ProfilesManager {
        let default_flags: Vec<String> = vec![
            "-pedantic",
            "-Wall",
            "-Wextra",
            "-Wcast-align",
            "-Wcast-qual",
            "-Wconversion",
            "-Wdisabled-optimization",
            "-Wmissing-include-dirs",
            "-Wmissing-noreturn",
            "-Wshadow",
            "-Wstack-protector",
            "-Wunreachable-code",
            "-Wfloat-equal",
            "-Wunused",
            "-Wswitch",
            "-Wswitch-enum",
            "-Winit-self",
            "-Wuninitialized",
            "-Wformat=2",
            "-Wformat-nonliteral",
            "-Wformat-security",
            "-Wformat-y2k",
            "-Winline",
            "-Wredundant-decls",
        ]
            .iter()
            .map(ToString::to_string)
            .collect();

        let mut default_profiles: HashMap<String, Rc<Profile>> = HashMap::new();

        let mut release_profile = Profile::empty("release");
        release_profile.compiler = "g++".to_string();
        release_profile.standard = "-std=c++17".to_string();
        release_profile.options = vec!["-O2", "-g0", "-Werror"]
            .iter()
            .map(ToString::to_string)
            .chain(default_flags.clone())
            .collect();

        let mut debug_profile = Profile::empty("debug");
        debug_profile.compiler = "g++".to_string();
        debug_profile.standard = "-std=c++17".to_string();
        debug_profile.options = vec!["-O0", "-g3", "-Werror"]
            .iter()
            .map(ToString::to_string)
            .chain(default_flags.clone())
            .collect();

        let mut release_unsafe_profile = Profile::empty("release-unsafe");
        release_unsafe_profile.compiler = "g++".to_string();
        release_unsafe_profile.standard = "-std=c++17".to_string();
        release_unsafe_profile.options = vec!["-O3", "-g0"]
            .iter()
            .map(ToString::to_string)
            .chain(default_flags.clone())
            .collect();

        let mut debug_unsafe_profile = Profile::empty("debug-unsafe");
        debug_unsafe_profile.compiler = "g++".to_string();
        debug_unsafe_profile.standard = "-std=c++17".to_string();
        debug_unsafe_profile.options = vec!["-O0", "-g3"]
            .iter()
            .map(ToString::to_string)
            .chain(default_flags.clone())
            .collect();

        let mut debug_asan_profile = Profile::empty("debug-asan");
        debug_asan_profile.compiler = "g++".to_string();
        debug_asan_profile.standard = "-std=c++17".to_string();
        debug_asan_profile.linking_options = vec![
            "-fsanitize=address",
            "-fsanitize=undefined",
            "-fsanitize=leak"
        ]
            .iter()
            .map(ToString::to_string)
            .collect();

        debug_asan_profile.options = vec![
            "-O0",
            "-g3",
            "-Werror",
            "-fsanitize=address",
            "-fsanitize=undefined",
            "-fsanitize=leak",
        ]
            .iter()
            .map(ToString::to_string)
            .chain(default_flags.clone())
            .collect();

        let mut debug_tsan_profile = Profile::empty("debug-tsan");
        debug_tsan_profile.compiler = "g++".to_string();
        debug_tsan_profile.standard = "-std=c++17".to_string();
        debug_tsan_profile
            .linking_options
            .push("-fsanitize=thread".to_string());
        debug_tsan_profile.options = vec!["-O0", "-g3", "-Werror", "-fsanitize=thread"]
            .iter()
            .map(ToString::to_string)
            .chain(default_flags.clone())
            .collect();

        if let Some(config) = config {
            if let toml::Value::Table(t) = config {
                for (key, value) in t {
                    match key.as_str() {
                        "release" => release_profile.fill_from_config(value),
                        "debug" => debug_profile.fill_from_config(value),
                        "release-unsafe" => release_unsafe_profile.fill_from_config(value),
                        "debug-unsafe" => debug_unsafe_profile.fill_from_config(value),
                        "debug-asan" => debug_asan_profile.fill_from_config(value),
                        "debug-tsan" => debug_tsan_profile.fill_from_config(value),
                        _ => continue
                    };
                    match Profile::from_config(key, value) {
                        Ok(profile) => {
                            default_profiles.insert(key.to_string(), Rc::new(profile));
                            ()
                        }
                        Err(error) => println!("Can't add {}: {:?}", key, error)
                    };
                }
            }
        }

        default_profiles.insert(String::from("release"), Rc::new(release_profile));
        default_profiles.insert(String::from("debug"), Rc::new(debug_profile));
        default_profiles.insert(String::from("release-unsafe"), Rc::new(release_unsafe_profile));
        default_profiles.insert(String::from("debug-unsafe"), Rc::new(debug_unsafe_profile));
        default_profiles.insert(String::from("debug-asan"), Rc::new(debug_asan_profile));
        default_profiles.insert(String::from("debug-tsan"), Rc::new(debug_tsan_profile));

        ProfilesManager {
            profiles: default_profiles,
        }
    }

    pub fn get(&self, profile_name: &str) -> Option<Rc<Profile>> {
        if let Some(profile) = self.profiles.get(profile_name) {
            Some(Rc::clone(profile))
        } else {
            None
        }
    }
}
