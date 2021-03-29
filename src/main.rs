// use clap::{App, AppSettings, Arg};
// use notify::{RecommendedWatcher, RecursiveMode, Watcher};
// use pyo3::prelude::*;
// use pyo3::types::IntoPyDict;
// use std::io;
// use std::io::Write;
// use std::path::Path;

// fn main() {
//     let matches = App::new("PySprint-CLI")
//         .setting(AppSettings::ColorAlways)
//         .setting(AppSettings::ColoredHelp)
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
//     // start a python interpreter
//     let gil = Python::acquire_gil();
//     let py = gil.python();
//     // with pysprint imported already
//     let locals = [("ps", py.import("pysprint")?)].into_py_dict(py);
//     let result = py.run(content, None, Some(&locals));
//     // print Python errors only, stay quiet when Ok(())
//     match result {
//         Err(ref err) => {
//             println!("Python error:\n{:?}", err);
//         }
//         _ => {}
//     }
//     Ok(())
// }

// fn watch<P: AsRef<Path>>(path: P) -> notify::Result<()> {
//     let (tx, rx) = std::sync::mpsc::channel();

//     let mut watcher: RecommendedWatcher = Watcher::new_immediate(move |res| tx.send(res).unwrap())?;

//     watcher.watch(path, RecursiveMode::NonRecursive)?;

//     for res in rx {
//         match res {
//             Ok(event) => {
//                 let ext = &event.paths[0].extension().unwrap();
//                 // println!("change in {:?}", &event.paths[0]);

//                 // stdout is frequently line-buffered by default so it is necessary
//                 // to flush() to ensure the output is emitted immediately
//                 io::stdout().flush().unwrap();

//                 if ext.to_str() == Some("py") {
//                     let content = std::fs::read_to_string(&event.paths[0])
//                         .expect("Something went wrong reading the file");
//                     // "clear" terminal
//                     print!("\x1B[2J\x1B[1;1H");
//                     exec_py(&content);
//                 }
//             }
//             Err(e) => println!("watch error: {:?}", e),
//         }
//     }
//     Ok(())
// }

use tera::{Tera, Context};
use lazy_static::lazy_static;
use pysprint_cli::codegen::write_tempfile;

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let tera = match Tera::new("src/templates/*") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        tera
    };
}

fn main() {

// Using the tera Context struct
    // let tera = match Tera::new("src/templates/*") {
    //     Ok(t) => t,
    //     Err(e) => {
    //         println!("Parsing error(s): {}", e);
    //         ::std::process::exit(1);
    //     }
    // };
    // println!("{:?}", tera);
    let mut context = Context::new();
    context.insert("methodname", "FFTMethod");
    context.insert("filename", "a");
    context.insert("filename2", "b");
    context.insert("filename3", "c");
    context.insert("skiprows", &8);
    context.insert("decimal", ",");
    context.insert("delimiter", ";");
    context.insert("meta_len", &6);
    // context.insert("slice_start", &5);
    // context.insert("slice_stop", &5);
    let result = TEMPLATES.render("boilerplate.py_t", &context);
    // let result = Tera::one_off("boilerplate.py_t", &context, true);
    // println!("{:?}", result.unwrap());
    write_tempfile("test", true, &result.unwrap());
}


