use clap::{App, Arg, AppSettings};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use pyo3::{py_run, prelude::*, PyErr};
use pyo3::types::IntoPyDict;
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
            println!("error: {:?}", e)
        }
    }
}

fn exec_py(content: &str) -> PyResult<()> {
    // clear terminal

    // print!("{esc}[2J{esc}[1;1H", esc = 27 as char);

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
        _ => {},
    }
    Ok(())
}

fn watch<P: AsRef<Path>>(path: P) -> notify::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher: RecommendedWatcher = Watcher::new_immediate(move |res| tx.send(res).unwrap())?;

    watcher.watch(path, RecursiveMode::NonRecursive)?;

    for res in rx {
        match res {
            Ok(event) => {
                let ext = &event.paths[0].extension().unwrap();
                println!("detected change in {:?}", &event.paths[0]);
                // println!("and the extension is {:?}", ext);
                if ext.to_str() == Some("py") {
                    let content = std::fs::read_to_string(&event.paths[0])
                        .expect("Something went wrong reading the file");
                    // println!("content is: {:?}", &content);
                    print!("\x1B[2J\x1B[1;1H");
                    exec_py(&content);
                }

                
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }
    Ok(())
}
