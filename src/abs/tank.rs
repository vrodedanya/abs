use std::fs;

use super::profiles_manager::ProfilesManager;
use super::section::Section;

#[derive(Debug)]
pub enum TankError {
    ConfigFileDoesntExist(String),
    WrongFormatOfToml(String),
    MandatoryLack(String),
    WrongTypeOfField(String),
    SectionError(String),
}

#[allow(unused)]
pub struct Tank {
    name: String,
    config: toml::Value,
    version: String, // todo Probably semver type?
    sections: Vec<Section>,

    profiles_manager: ProfilesManager,
}

#[allow(unused)]
impl Tank {
    pub fn new(config_name: &str) -> Result<Tank, TankError> {
        let mut config: toml::Value = toml::from_str(
            &fs::read_to_string(config_name)
                .map_err(|err| TankError::ConfigFileDoesntExist(err.to_string()))?,
        )
        .map_err(|err| TankError::WrongFormatOfToml(err.to_string()))?;

        let tank_config = config
            .get("tank")
            .ok_or_else(|| {
                TankError::MandatoryLack("Can't find 'tank' table which is mandatory".to_string())
            })?
            .clone();
        let name_of_tank = tank_config
            .get("name")
            .ok_or_else(|| TankError::MandatoryLack("Can't find name of tank".to_string()))?;
        let version_of_tank = tank_config
            .get("version")
            .ok_or_else(|| TankError::MandatoryLack("Can't find version of tank".to_string()))?;

        let mut sections_config = config.get_mut("sections");

        let mut sections: Vec<Section> = vec![];

        if let Some(sections_config) = sections_config {
            if let toml::Value::Table(t) = sections_config {
                for (key, value) in t {
                    sections.push(
                        Section::new(key.to_string(), &value)
                            .map_err(|err| TankError::SectionError(format!("{:#?}", err)))?,
                    );
                }
            }
        }

        let tank = Tank {
            name: name_of_tank
                .as_str()
                .ok_or_else(|| TankError::WrongTypeOfField("Can't find name of tank".to_string()))?
                .to_string(),
            config: config.clone(),
            version: version_of_tank
                .as_str()
                .ok_or_else(|| {
                    TankError::WrongTypeOfField("Can't find version of tank".to_string())
                })?
                .to_string(),
            sections,
            profiles_manager: ProfilesManager::new(config.get("profiles")),
        };
        return Ok(tank);
    }

    pub fn check(&self, profile_name: &str) -> bool {
        if let Some(profile) = self.profiles_manager.get(profile_name) {
            self.sections.iter().all(|section| section.check(profile))
        } else {
            println!("'{}' doesn't exist", profile_name);
            return false;
        }
    }

    pub fn build(&self, profile_name: &str) -> bool {
        if let Some(profile) = self.profiles_manager.get(profile_name) {
            self.sections.iter().all(|section| section.build(profile))
        } else {
            println!("'{}' doesn't exist", profile_name);
            return false;
        }
    }

    pub fn run(&self, profile_name: &str) -> bool {
        if let Some(profile) = self.profiles_manager.get(profile_name) {
            self.sections.iter().all(|section| section.run(profile))
        } else {
            println!("'{}' doesn't exist", profile_name);
            return false;
        }
    }

    pub fn print_sections(&self) {
        for section in &self.sections {
            println!("{:#?}\n", section);
        }
    }
}
