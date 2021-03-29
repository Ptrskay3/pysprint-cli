fn read_yaml(file: &str) -> Result<serde_yaml::Value, Box<dyn std::error::Error>> {
    let f = std::fs::File::open(file)?;
    Ok(serde_yaml::from_reader(f)?)
}
use std::collections::HashMap;

pub fn parse(file: &str) -> (HashMap<String, Box<f64>>, HashMap<String, String>) {
    let yaml_file = read_yaml(file).unwrap();

    // options that can be represented as a number
    let mut number_options: HashMap<String, Box<f64>> = HashMap::new();
    // options that can be represented as text
    let mut text_options: HashMap<String, String> = HashMap::new();

    // parsing the "load_options section"
    for feature in yaml_file["load_options"].as_sequence().iter() {
        for (_, entities) in feature.iter().enumerate() {
            let load_option: serde_yaml::Value = serde_yaml::to_value(entities).unwrap();
            if let serde_yaml::Value::Mapping(options) = load_option {
                for option in options.iter() {
                    match option {
                        (serde_yaml::Value::String(key), serde_yaml::Value::Number(val)) => {
                            number_options.insert(key.to_string(), Box::new(val.as_f64().unwrap()));
                        }
                        (serde_yaml::Value::String(key), serde_yaml::Value::String(val)) => {
                            text_options.insert(key.to_string(), val.to_string());
                        }
                        _ => panic!(
                            "yaml contains values that are unknown in this context: {:?}",
                            option
                        ),
                    }
                }
            }
        }
    }
    (number_options, text_options)
}
