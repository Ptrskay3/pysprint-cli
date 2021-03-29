use clap::{App, AppSettings, Arg};
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use pysprint_cli::{
    codegen::{render_template, write_tempfile},
    parser::parse,
};
use std::io;
use std::io::Write;
use std::path::Path;
use std::time::Duration;

fn main() {
    let matches = App::new("PySprint-CLI")
        .setting(AppSettings::ColorAlways)
        .setting(AppSettings::ColoredHelp)
        .version("0.28.0")
        .author("Péter Leéh")
        .help("PySprint watching engine for interferogram evaluation")
        .arg(
            Arg::with_name("path")
                .short("p")
                .long("path")
                .value_name("FILE")
                .help("set up the filepath to watch")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("CONFIG")
                .help("the config file to use")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("persist")
                .long("persist")
                .value_name("PERSIST")
                .help("persist the evaluation files")
                .takes_value(false),
        )
        .get_matches();

    if let Some(filepath) = matches.value_of("path") {
        println!("PySprint watch mode active. Start recording/changing files..");
        let config_file = matches.value_of("config").unwrap_or("eval.yaml");
        if let Err(e) = watch(filepath, config_file, matches.is_present("persist")) {
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
    if let Err(ref err) = result {
        println!("Python error:\n{:?}", err);
    }
    Ok(())
}

fn watch<P: AsRef<Path> + Copy>(path: P, config_file: &str, persist: bool) -> notify::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher = watcher(tx, Duration::from_millis(200)).unwrap();
    // let mut watcher: RecommendedWatcher = Watcher::new_immediate(move |res| tx.send(res).unwrap())?;
    watcher.watch(path, RecursiveMode::NonRecursive)?;

    // we need to append the filepath to the template, because python also runs from *here*.
    let fpath = &path.as_ref().to_str().unwrap();
    let (
        numeric_config,
        string_config,
        boolean_config,
        before_evaluate_triggers,
        after_evaluate_triggers,
    ) = parse(&format!("{}/{}", fpath, config_file));

    loop {
        match rx.recv() {
            Ok(event) => {
                match event {
                    // only trigger on Write and Create events..
                    DebouncedEvent::Write(e) | DebouncedEvent::Create(e) => {
                        // get the extension, we need to see whether we care
                        let ext = &e.extension();

                        match ext {
                            Some(value) => {
                                if value.to_str() == Some("trt") {
                                    // TODO: filter files to skip

                                    // clear terminal on rerun
                                    print!("\x1B[2J\x1B[1;1H");
                                    // stdout is frequently line-buffered by default so it is necessary
                                    // to flush() to ensure the output is emitted immediately
                                    io::stdout().flush().unwrap();

                                    // render the code that needs to be executed
                                    let code = render_template(
                                        &e.file_name().unwrap().to_str().unwrap(),
                                        fpath,
                                        &string_config,
                                        &numeric_config,
                                        &boolean_config,
                                        &before_evaluate_triggers,
                                        &after_evaluate_triggers,
                                    );

                                    // write to file the code if needed
                                    if persist {
                                        let _ = write_tempfile(
                                            &e.file_stem().unwrap().to_str().unwrap(),
                                            code.as_ref().unwrap(),
                                        );
                                    }

                                    // execute it
                                    let _ = exec_py(&code.unwrap());
                                }
                            }
                            None => {} // if there's no extension, we probably should do nothing
                        }
                    }
                    _ => {}
                }
            }

            Err(e) => println!("watch error: {:?}", e),
        }
    }
}
