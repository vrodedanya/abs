use std::fs;

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
    sections: Vec<Section>
}

#[allow(unused)]
impl Tank {
    pub fn new(config_name: &str) -> Result<Tank, TankError> {
        let mut config: toml::Value = toml::from_str(&fs::read_to_string(config_name)
            .map_err(|err|TankError::ConfigFileDoesntExist(err.to_string()))?)
            .map_err(|err|TankError::WrongFormatOfToml(err.to_string()))?;

        let tank_config = config.get("tank")
            .ok_or_else(||TankError::MandatoryLack("Can't find 'tank' table which is mandatory".to_string()))?.clone();
        let name_of_tank = tank_config.get("name")
            .ok_or_else(||TankError::MandatoryLack("Can't find name of tank".to_string()))?;
        let version_of_tank = tank_config.get("version")
            .ok_or_else(||TankError::MandatoryLack("Can't find version of tank".to_string()))?;

        let mut sections_config = config.get_mut("sections");

        let mut sections: Vec<Section> = vec![];

        if sections_config.is_some() {
            let sections_config = sections_config.unwrap();
            if let toml::Value::Table(t) = sections_config {
                for (key, value) in t {
                    println!("{}", key);
                    sections.push(Section::new(key.to_string(), &value)
                        .map_err(|err|TankError::SectionError(format!("{:#?}", err)))?);
                }
            }
        }

        let tank = Tank {
            name: name_of_tank.as_str().ok_or_else(||TankError::WrongTypeOfField("Can't find name of tank".to_string()))?.to_string(),
            config: config.clone(),
            version: version_of_tank.as_str().ok_or_else(||TankError::WrongTypeOfField("Can't find version of tank".to_string()))?.to_string(),
            sections
        };
        return Ok(tank);
    }

    pub fn check(&self) -> bool {
        self.sections.iter().all(|section|section.check())
    }

    pub fn print_sections(&self) {
        for section in &self.sections {
            println!("{:#?}\n", section);
        }
    }
}