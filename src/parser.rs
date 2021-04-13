use std::collections::HashMap;

#[derive(Debug)]
pub struct EvaluateOptions {
    pub number_options: HashMap<String, Box<f64>>,
    pub text_options: HashMap<String, String>,
    pub bool_options: HashMap<String, Box<bool>>,
}

#[derive(Debug)]
pub struct IntermediateHooks {
    pub before_evaluate_triggers: Vec<String>,
    pub after_evaluate_triggers: Vec<String>,
}

#[derive(Debug)]
pub struct FilePatternOptions {
    pub exclude_patterns: Vec<String>,
    pub skip_files: Vec<String>,
    pub extensions: Vec<String>,
}

fn read_yaml(file: &str) -> Result<serde_yaml::Value, Box<dyn std::error::Error>> {
    let f = std::fs::File::open(file)?;
    Ok(serde_yaml::from_reader(f)?)
}

pub fn parse(file: &str) -> (EvaluateOptions, IntermediateHooks, FilePatternOptions) {
    let yaml_file = read_yaml(file).unwrap();

    let mut number_options: HashMap<String, Box<f64>> = HashMap::new();
    let mut text_options: HashMap<String, String> = HashMap::new();
    let mut bool_options: HashMap<String, Box<bool>> = HashMap::new();
    let mut before_evaluate_triggers: Vec<String> = Vec::new();
    let mut after_evaluate_triggers: Vec<String> = Vec::new();

    // options that relate to the loading optios
    let mut exclude_patterns: Vec<String> = Vec::new();
    let mut skip_files: Vec<String> = Vec::new();
    let mut extensions: Vec<String> = Vec::new();

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
                            (serde_yaml::Value::String(key), serde_yaml::Value::Sequence(seq)) => {
                                match key.as_str() {
                                    "exclude_patterns" => {
                                        seq.iter()
                                            .filter_map(|d| match d {
                                                serde_yaml::Value::String(string) => {
                                                    Some(string.to_owned())
                                                }
                                                _ => None,
                                            })
                                            .collect::<Vec<String>>()
                                            .as_slice()
                                            .clone_into(&mut exclude_patterns);
                                    }
                                    "skip" => {
                                        seq.iter()
                                            .filter_map(|d| match d {
                                                serde_yaml::Value::String(string) => {
                                                    Some(string.to_owned())
                                                }
                                                _ => None,
                                            })
                                            .collect::<Vec<String>>()
                                            .as_slice()
                                            .clone_into(&mut skip_files);
                                    }
                                    "extensions" => {
                                        seq.iter()
                                            .filter_map(|d| match d {
                                                serde_yaml::Value::String(string) => {
                                                    Some(string.to_owned())
                                                }
                                                _ => None,
                                            })
                                            .collect::<Vec<String>>()
                                            .as_slice()
                                            .clone_into(&mut extensions);
                                    }
                                    _ => {}
                                }
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
                before_evaluate_triggers.push(cmd.to_string());
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
                    "mm" => text_options
                        .insert(String::from("methodname"), String::from("MinMaxMethod")),
                    "spp" => {
                        text_options.insert(String::from("methodname"), String::from("SPPMethod"))
                    }
                    "cff" => text_options
                        .insert(String::from("methodname"), String::from("CosFitMethod")),

                    _ => panic!("method named {:?} is not implemented", cmd),
                };
            }
        }
    }

    for commands in yaml_file["method_details"].as_sequence().iter() {
        for command in commands.iter() {
            match command {
                serde_yaml::Value::String(cmd) => {
                    bool_options.insert(cmd.to_string(), Box::new(true));
                }
                serde_yaml::Value::Mapping(options) => {
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
                _ => {}
            }
        }
    }

    (
        EvaluateOptions {
            number_options,
            text_options,
            bool_options,
        },
        IntermediateHooks {
            before_evaluate_triggers,
            after_evaluate_triggers,
        },
        FilePatternOptions {
            exclude_patterns,
            skip_files,
            extensions,
        },
    )
}
