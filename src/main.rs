use clap::{App, AppSettings, Arg};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use pysprint_cli::{codegen::render_template, parser::parse};
use std::collections::HashMap;
use std::io;
use std::io::Write;
use std::path::Path;

fn main() {
    let matches = App::new("PySprint-CLI")
        .setting(AppSettings::ColorAlways)
        .setting(AppSettings::ColoredHelp)
        .version("0.28.0")
        .author("Péter Leéh")
        .help("Powerful watching engine for interferogram evaluation")
        .arg(
            Arg::with_name("path")
                .short("p")
                .long("path")
                .value_name("FILE")
                .help("set up the filepath to watch")
                .takes_value(true),
        )
        .get_matches();

    if let Some(filepath) = matches.value_of("path") {
        println!("PySprint watch mode active..");
        if let Err(e) = watch(filepath) {
            println!("error watching..: {:?}", e)
        }
    }
}

fn exec_py(content: &str) -> PyResult<()> {
    // start a python interpreter
    let gil = Python::acquire_gil();
    let py = gil.python();
    // with pysprint imported already
    let locals = [("ps", py.import("pysprint")?)].into_py_dict(py);
    let result = py.run(content, None, Some(&locals));
    // print Python errors only, stay quiet when Ok(())
    match result {
        Err(ref err) => {
            println!("Python error:\n{:?}", err);
        }
        _ => {}
    }
    Ok(())
}

fn watch<P: AsRef<Path>>(path: P) -> notify::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher: RecommendedWatcher = Watcher::new_immediate(move |res| tx.send(res).unwrap())?;

    watcher.watch(path, RecursiveMode::NonRecursive)?;

    let (num, text) = parse("./example/eval.yaml");

    for res in rx {
        match res {
            Ok(event) => {
                let ext = &event.paths[0].extension();
                // println!("ext is{:?}", ext);
                // println!("change in {:?}", &event.paths[0]);

                // stdout is frequently line-buffered by default so it is necessary
                // to flush() to ensure the output is emitted immediately
                io::stdout().flush().unwrap();
                match ext {
                    Some(value) => {
                        print!("\x1B[2J\x1B[1;1H");
                        if value.to_str() == Some("trt") {
                            let code = render_template(
                                &event.paths[0].file_name().unwrap().to_str().unwrap(),
                                &text,
                                &num,
                            );
                            // println!("{:?}", code);
                            exec_py(&code.unwrap());
                        }
                    }
                    None => {
                        println!("{:?}", event);
                    } // some unknown event..
                }
                // if ext.unwrap().to_str() == Some("trt") {
                    
                // let content = std::fs::read_to_string(&event.paths[0])
                //     .expect("Something went wrong reading the file");
                // "clear" terminal

                // println!("{:?}{:?}", num, text);

                // print!("\x1B[2J\x1B[1;1H");

                // }
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }
    Ok(())
}
