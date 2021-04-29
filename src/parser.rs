use crate::deserialize::Config;

pub fn parse(file: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let r = std::fs::File::open(file)?;
    let config: Config = serde_yaml::from_reader(r)?;
    Ok(config)
}
