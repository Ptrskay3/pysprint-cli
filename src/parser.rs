use std::collections::HashMap;

// // TODO
// pub struct EvaluateOptions {
//     numeric: HashMap<String, Box<f64>>,
//     textual: HashMap<String, String>,
//     boolean: HashMap<String, Box<bool>>
// }

// // TODO
// pub struct IntermediateHooks {
//     before_evaulate_triggers: Vec<String>,
//     after_evaulate_triggers: Vec<String>
// }

fn read_yaml(file: &str) -> Result<serde_yaml::Value, Box<dyn std::error::Error>> {
    let f = std::fs::File::open(file)?;
    Ok(serde_yaml::from_reader(f)?)
}

pub fn parse(
    file: &str,
) -> (
    HashMap<String, Box<f64>>,
    HashMap<String, String>,
    HashMap<String, Box<bool>>,
    Vec<String>,
    Vec<String>,
) {
    let yaml_file = read_yaml(file).unwrap();

    // options that can be represented as a number
    let mut number_options: HashMap<String, Box<f64>> = HashMap::new();
    // options that can be represented as text
    let mut text_options: HashMap<String, String> = HashMap::new();
    // options that can be represented as boolean
    let mut bool_options: HashMap<String, Box<bool>> = HashMap::new();
    // trigger before evaluate
    let mut before_evaulate_triggers: Vec<String> = Vec::new();
    // trigger after evaluate
    let mut after_evaluate_triggers: Vec<String> = Vec::new();

    // parsing the standard sections
    for section in vec!["load_options", "preprocess", "evaluate"].iter() {
        for feature in yaml_file[section].as_sequence().iter() {
            for entities in feature.iter() {
                let load_option: serde_yaml::Value = serde_yaml::to_value(entities).unwrap();
                if let serde_yaml::Value::Mapping(options) = load_option {
                    for option in options.iter() {
                        match option {
                            (serde_yaml::Value::String(key), serde_yaml::Value::Number(val)) => {
                                number_options
                                    .insert(key.to_string(), Box::new(val.as_f64().unwrap()));
                            }
                            (serde_yaml::Value::String(key), serde_yaml::Value::String(val)) => {
                                text_options.insert(key.to_string(), val.to_string());
                            }
                            (serde_yaml::Value::String(key), serde_yaml::Value::Bool(val)) => {
                                bool_options.insert(key.to_string(), Box::new(*val));
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
    }
    // parsing the "before_evaluate" section for subcommands to run
    for commands in yaml_file["before_evaluate"].as_sequence().iter() {
        for command in commands.iter() {
            if let serde_yaml::Value::String(cmd) = command {
                before_evaulate_triggers.push(cmd.to_string());
            }
        }
    }

    // parsing the "after_evaluate" section for subcommands to run
    for commands in yaml_file["after_evaluate"].as_sequence().iter() {
        for command in commands.iter() {
            if let serde_yaml::Value::String(cmd) = command {
                after_evaluate_triggers.push(cmd.to_string());
            }
        }
    }

    // getting the method section
    for commands in yaml_file["method"].as_sequence().iter() {
        for command in commands.iter() {
            if let serde_yaml::Value::String(cmd) = command {
                match cmd.to_string().as_str() {
                    "fft" => {
                        text_options.insert(String::from("methodname"), String::from("FFTMethod"))
                    }
                    "wft" => {
                        text_options.insert(String::from("methodname"), String::from("WFTMethod"))
                    }
                    _ => panic!("method named {:?} is not implemented", cmd),
                };
            }
        }
    }

    (
        number_options,
        text_options,
        bool_options,
        before_evaulate_triggers,
        after_evaluate_triggers,
    )
}
