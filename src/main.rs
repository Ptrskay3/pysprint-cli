use clap::{App, Arg};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use std::path::Path;

fn main() {
    let matches = App::new("PySprint-CLI")
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
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);

    // start a python interpreter
    let gil = Python::acquire_gil();
    let py = gil.python();
    let globals = [("ps", py.import("pysprint")?)].into_py_dict(py);
    // run a test if it works
    // println!("{:?}", ps.get("__version__")?);
    // let globals = PyDict::new(py);
    let _result = py.eval(content, None, Some(&globals));
    println!("{:?}", _result);
    Ok(())
}

fn watch<P: AsRef<Path>>(path: P) -> notify::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher: RecommendedWatcher = Watcher::new_immediate(move |res| tx.send(res).unwrap())?;

    watcher.watch(path, RecursiveMode::NonRecursive)?;

    for res in rx {
        match res {
            Ok(event) => {
                println!("file was changed in path {:?}", &event.paths[0]);
                let content = std::fs::read_to_string(&event.paths[0])
                    .expect("Something went wrong reading the file");
                // println!("content is: {:?}", &content);
                exec_py(&content);
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }
    Ok(())
}
