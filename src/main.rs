// use clap::{App, Arg};
// use notify::{RecommendedWatcher, RecursiveMode, Watcher};
// use pyo3::prelude::*;
// use pyo3::types::IntoPyDict;
// use std::path::Path;

// fn main() {
//     let matches = App::new("PySprint-CLI")
//         .version("0.28.0")
//         .author("Péter Leéh")
//         .help("Powerful watching engine for interferogram evaluation")
//         .arg(
//             Arg::with_name("path")
//                 .short("p")
//                 .long("path")
//                 .value_name("FILE")
//                 .help("set up the filepath to watch")
//                 .takes_value(true),
//         )
//         .get_matches();

//     if let Some(filepath) = matches.value_of("path") {
//         println!("PySprint watch mode active..");
//         if let Err(e) = watch(filepath) {
//             println!("error: {:?}", e)
//         }
//     }
// }

// fn exec_py(content: &str) -> PyResult<()> {
//     // clear terminal
//     print!("{esc}[2J{esc}[1;1H", esc = 27 as char);

//     // start a python interpreter
//     let gil = Python::acquire_gil();
//     let py = gil.python();
//     let globals = [("ps", py.import("pysprint")?)].into_py_dict(py);
//     // run a test if it works
//     // println!("{:?}", ps.get("__version__")?);
//     // let globals = PyDict::new(py);
//     let _result = py.eval(content, Some(&globals), None);
//     Ok(())
// }

// fn watch<P: AsRef<Path>>(path: P) -> notify::Result<()> {
//     let (tx, rx) = std::sync::mpsc::channel();

//     let mut watcher: RecommendedWatcher = Watcher::new_immediate(move |res| tx.send(res).unwrap())?;

//     watcher.watch(path, RecursiveMode::Recursive)?;

//     for res in rx {
//         match res {
//             Ok(event) => {
//                 // println!("file was changed in path {:?}", &event.paths[0]);
//                 let content = std::fs::read_to_string(&event.paths[0])
//                     .expect("Something went wrong reading the file");
//                 // println!("content is: {:?}", &content);
//                 let _ = exec_py(&content);
//             }
//             Err(e) => println!("watch error: {:?}", e),
//         }
//     }
//     Ok(())
// }

fn read_yaml(file: &str) -> Result<serde_yaml::Value, Box<dyn std::error::Error>> {
    let f = std::fs::File::open(file)?;
    Ok(serde_yaml::from_reader(f)?)
}

fn main() {
    let a = read_yaml("example/eval.yaml").unwrap();
    for feature in a["load_options"].as_sequence().iter() {
        for (i, entities) in feature.iter().enumerate() {
            let load_option: serde_yaml::Value = serde_yaml::to_value(entities).unwrap();
            match load_option {
                // serde_yaml::Value::String(o) => println!("string {:?}", o),
                serde_yaml::Value::Mapping(option) => {
                    for op in option.iter() {
                        match op {
                            (serde_yaml::Value::String(key), serde_yaml::Value::Number(val)) => println!("k:{:?}, v:{:?}", key, val),
                            (serde_yaml::Value::String(key), serde_yaml::Value::String(val)) => println!("k:{:?}, v:{:?}", key, val),
                            _ => panic!("yaml contains values that are unknown in this context: {:?}", op),
                        }
                    }
                }
                _ => {},
            }
            // println!("{:?}", load_option);
        }
    }
    // }
    // println!("{:?}", b);
    // let featues = &a["featues"];
    // println!("{:?}", a["featues"]);
}
