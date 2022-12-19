use std::collections::HashMap;

use super::profile::Profile;

#[derive(Clone)]
#[allow(unused)]
pub struct ProfilesManager {
    profiles: HashMap<String, Profile>
}

#[allow(unused)]
impl ProfilesManager {
    pub fn new(config: Option<&toml::Value>) -> ProfilesManager {

        let mut default_profiles: HashMap<String, Profile> = HashMap::new();

        let mut release_profile = Profile::new("release");
        release_profile.compiler = "g++".to_string();
        release_profile.standard = "-std=c++17".to_string();
        release_profile.options = vec![
            "-O2",
            "-g3",
            "-Werror",
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
            "-Wredundant-decls"
        ].iter().map(ToString::to_string).collect();
        let mut debug_profile = Profile::new("debug");
        let mut release_unsafe_profile = Profile::new("release-unsafe");
        let mut debug_unsafe_profile = Profile::new("debug-unsafe");
        let mut debug_asan_profile = Profile::new("debug-asan");
        let mut debug_tsan_profile = Profile::new("debug-tsan");
/* 
        if let Some(config) = config {
            if let toml::Value::Table(t) = config {
                for (key, value) in t {
                    println!("{} {:#?}", key, value);
                }
            }
        } */

        default_profiles.insert(String::from("release"), release_profile);
        default_profiles.insert(String::from("debug"), debug_profile);
        default_profiles.insert(String::from("release-unsafe"), release_unsafe_profile);
        default_profiles.insert(String::from("debug-unsafe"), debug_unsafe_profile);
        default_profiles.insert(String::from("debug-asan"), debug_asan_profile);
        default_profiles.insert(String::from("debug-tsan"), debug_tsan_profile);

        ProfilesManager { profiles: default_profiles }
    }

    pub fn get(&self, profile_name: &str) -> Option<&Profile> {
        self.profiles.get(profile_name)
    }
}