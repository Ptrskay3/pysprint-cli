use clap::{App, AppSettings, Arg, SubCommand};
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use pysprint_cli::{
    codegen::{maybe_write_default_yaml, render_template, write_tempfile},
    parser::parse,
};
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::Path;
use std::time::Duration;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

fn main() {
    let matches = App::new("PySprint-CLI")
        .setting(AppSettings::ColorAlways)
        .setting(AppSettings::ColoredHelp)
        .version("0.28.0")
        .author("Péter Leéh")
        .help("PySprint watching engine for interferogram evaluation")
        .subcommand(
            SubCommand::with_name("watch")
                .arg(
                    Arg::with_name("path")
                        .short("p")
                        .long("path")
                        .value_name("FILE")
                        .help("set up the filepath to watch")
                        .takes_value(true)
                        .required(true)
                        .index(1),
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
                    Arg::with_name("result")
                        .short("r")
                        .long("result")
                        .value_name("RESULT")
                        .help("the file to write results")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("persist")
                        .long("persist")
                        .value_name("PERSIST")
                        .help("persist the evaluation files")
                        .takes_value(false),
                )
                .arg(
                    Arg::with_name("verbosity")
                        .short("v")
                        .help("increase the verbosity level of results")
                        .multiple(true)
                        .takes_value(false),
                ),
        )
        .subcommand(SubCommand::with_name("audit"))
        .get_matches();

    if matches.subcommand_matches("audit").is_some() {
        todo!();
    }

    if let Some(matches) = matches.subcommand_matches("watch") {
        let verbosity: u8 = match matches.occurrences_of("verbosity") {
                0 => 0,
                1 | _ => 1,
        };        
        if let Some(filepath) = matches.value_of("path") {
            println!("[INFO] PySprint watch mode starting.");
            let config_file = matches.value_of("config").unwrap_or("eval.yaml");
            let config_filepath = Path::new(&filepath).join(config_file);
            if !config_filepath.exists() {
                maybe_write_default_yaml(&filepath);
            }
            let result_file = matches.value_of("result").unwrap_or("results.json");
            let result_filepath = Path::new(&filepath).join(result_file);
            if !result_file_is_present(&result_filepath).unwrap_or(true) {
                create_results_file(&result_filepath.into_os_string().to_str().unwrap()).unwrap();
            }
            
            println!("[INFO] Watch started..");

            if let Err(e) = watch(
                filepath,
                config_file,
                matches.is_present("persist"),
                result_file,
                verbosity,
            ) {
                println!("[ERRO] error watching..: {:?}", e)
            }
        }
    }
}

fn result_file_is_present<P: AsRef<Path>>(
    result_filepath: P,
) -> Result<bool, Box<dyn std::error::Error>> {
    if result_filepath.as_ref().exists() {
        let mut stdout = StandardStream::stdout(ColorChoice::Always);
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))?;
        let warning = format!(
            "[WARN] The result file named {:?} already exists. Its contents might be overwritten.",
            result_filepath.as_ref()
        );
        writeln!(&mut stdout, "{}", warning)?;
        let _ = WriteColor::reset(&mut stdout);
        Ok(true)
    } else {
        println!(
            "[INFO] Created {:?} result file.",
            result_filepath.as_ref().file_name().unwrap()
        );
        Ok(false)
    }
}

fn create_results_file(filename: &str) -> std::io::Result<()> {
    let mut file = File::create(filename)?;
    file.write_all(b"{ }")?;
    Ok(())
}

fn exec_py(content: &str) -> PyResult<()> {
    // start a python interpreter
    let gil = Python::acquire_gil();
    let py = gil.python();

    // with the required packages imported already
    let locals = [
        ("np", py.import("numpy")?),
        ("ps", py.import("pysprint")?),
        ("plt", py.import("matplotlib.pyplot")?),
    ]
    .into_py_dict(py);
    let result = py.run(content, None, Some(&locals));

    // print Python errors only, stay quiet when Ok(())
    if let Err(ref err) = result {
        let py_error = format!("[ERRO] Python error:\n{:?}", err);
        println!("{}", py_error);
        let _ = py.check_signals()?;
    }
    Ok(())
}

fn watch<P: AsRef<Path> + Copy>(
    path: P,
    config_file: &str,
    persist: bool,
    result_file: &str,
    verbosity: u8,
) -> notify::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher = watcher(tx, Duration::from_millis(100)).unwrap();
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
                                    // to flush() to ensure the clear above is emitted immediately
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
                                        &result_file,
                                        verbosity,
                                    );

                                    // write the generated code if needed
                                    if persist {
                                        let _ = write_tempfile(
                                            &e.file_stem().unwrap().to_str().unwrap(),
                                            code.as_ref().unwrap(),
                                            fpath,
                                        );
                                    }

                                    // execute it
                                    let _ = exec_py(&code.unwrap());
                                }
                            }
                            None => {} // if there's no extension, we probably should do nothing
                        }
                    }
                    _ => {} // there is something wrong with the event, probably we also should skip
                }
            }

            Err(e) => println!("watch error: {:?}", e),
        }
    }
}
