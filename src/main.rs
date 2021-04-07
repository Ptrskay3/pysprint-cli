use clap::{App, AppSettings, Arg, SubCommand};
use indicatif::{ProgressStyle, ProgressIterator, ProgressBar};
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use pysprint_cli::{
    audit::get_files,
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
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    let matches = App::new("PySprint-CLI")
        .setting(AppSettings::ColorAlways)
        .setting(AppSettings::ColoredHelp)
        .version("0.29.0")
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
        .subcommand(
            SubCommand::with_name("audit").arg(
                Arg::with_name("path")
                    .short("p")
                    .long("path")
                    .value_name("FILE")
                    .help("set up the filepath to watch")
                    .takes_value(true)
                    .required(true)
                    .index(1),
            ),
        )
        .get_matches();

    if let Some(cmd) = matches.subcommand_matches("audit") {
        if let Some(filepath) = cmd.value_of("path") {
            let config_file = matches.value_of("config").unwrap_or("eval.yaml");
            let config_filepath = Path::new(&filepath).join(config_file);
            if !config_filepath.exists() {
                maybe_write_default_yaml(&filepath);
            }

            let result_file = matches.value_of("result").unwrap_or("results.json");
            let result_filepath = Path::new(&filepath).join(result_file);
            if !result_file_is_present(&result_filepath, &mut stdout).unwrap_or(true) {
                create_results_file(&result_filepath.into_os_string().to_str().unwrap()).unwrap();
            }

            audit(&mut stdout, filepath, config_file, result_file);
        }
    }

    if let Some(matches) = matches.subcommand_matches("watch") {
        let verbosity: u8 = match matches.occurrences_of("verbosity") {
            0 => 0,
            _ => 1,
        };
        if let Some(filepath) = matches.value_of("path") {
            if let Err(e) = writeln!(stdout, "[INFO] PySprint watch mode starting.") {
                println!("Error writing to stdout: {}", e);
            }
            let config_file = matches.value_of("config").unwrap_or("eval.yaml");
            let config_filepath = Path::new(&filepath).join(config_file);
            if !config_filepath.exists() {
                maybe_write_default_yaml(&filepath);
            }
            let result_file = matches.value_of("result").unwrap_or("results.json");
            let result_filepath = Path::new(&filepath).join(result_file);
            if !result_file_is_present(&result_filepath, &mut stdout).unwrap_or(true) {
                create_results_file(&result_filepath.into_os_string().to_str().unwrap()).unwrap();
            }

            if let Err(e) = writeln!(stdout, "[INFO] Watch started..") {
                println!("Error writing to stdout: {}", e);
            }

            if let Err(e) = watch(
                filepath,
                config_file,
                matches.is_present("persist"),
                result_file,
                verbosity,
                &mut stdout,
            ) {
                if let Err(e) = writeln!(stdout, "[ERROR] error watching..: {:?}", e) {
                    println!("Error writing to stdout: {}", e);
                }
            }
        }
    }
}

fn result_file_is_present<P: AsRef<Path>>(
    result_filepath: P,
    stdout: &mut StandardStream,
) -> Result<bool, Box<dyn std::error::Error>> {
    if result_filepath.as_ref().exists() {
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))?;
        let warning = format!(
            "[WARN] The result file named {:?} already exists. Its contents might be overwritten.",
            result_filepath.as_ref()
        );
        if let Err(e) = writeln!(stdout, "{}", warning) {
            println!("Error writing to stdout: {}", e);
        }
        let _ = WriteColor::reset(stdout);
        Ok(true)
    } else {
        if let Err(e) = writeln!(
            stdout,
            "[INFO] Created {:?} result file.",
            result_filepath.as_ref().file_name().unwrap()
        ) {
            println!("Error writing to stdout: {}", e);
        }
        Ok(false)
    }
}

fn create_results_file(filename: &str) -> std::io::Result<()> {
    let mut file = File::create(filename)?;
    file.write_all(b"{ }")?;
    Ok(())
}

fn exec_py(content: &str, stdout: &mut StandardStream) -> PyResult<()> {
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
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
        let py_error = format!("[ERRO] Python error:\n{:?}", err);
        if let Err(e) = writeln!(stdout, "{}", py_error) {
            println!("Error writing to stdout: {}", e);
        }
        let _ = py.check_signals()?;
        let _ = WriteColor::reset(stdout);
    }
    Ok(())
}

fn audit(stdout: &mut StandardStream, filepath: &str, config_file: &str, result_file: &str) {
    let (
        numeric_config,
        string_config,
        boolean_config,
        before_evaluate_triggers,
        after_evaluate_triggers,
        file_pattern_options,
    ) = parse(&format!("{}/{}", filepath, config_file));

    let files = get_files(filepath, &file_pattern_options).unwrap();

    let bar = ProgressBar::new(files.len() as u64);
    
    bar.set_style(ProgressStyle::default_bar()
    .template("{prefix:>16.green} [{bar:57}] {pos}/{len} {msg}"));

    bar.set_prefix("Processing files");

    for file in files.iter() {
        bar.inc(1);

        // render the code that needs to be executed
        let code = render_template(
            file.as_path().file_name().unwrap().to_str().unwrap(),
            filepath,
            &string_config,
            &numeric_config,
            &boolean_config,
            &before_evaluate_triggers,
            &after_evaluate_triggers,
            &result_file,
            0,
        );

        // write the generated code if needed

        let _ = write_tempfile(
            file.as_path().file_stem().unwrap().to_str().unwrap(),
            code.as_ref().unwrap(),
            filepath,
        );

        // execute it
        let _ = exec_py(&code.unwrap(), stdout);
    }
    bar.finish_with_message("Check `results.json`.");
}

fn watch<P: AsRef<Path> + Copy>(
    path: P,
    config_file: &str,
    persist: bool,
    result_file: &str,
    verbosity: u8,
    stdout: &mut StandardStream,
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
        file_pattern_options,
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
                                if file_pattern_options
                                    .extensions
                                    .contains(&value.to_str().unwrap().to_owned())
                                {
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
                                    let _ = exec_py(&code.unwrap(), stdout);
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
