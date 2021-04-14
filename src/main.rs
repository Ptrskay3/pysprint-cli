use clap::{App, AppSettings, Arg, SubCommand};
use pysprint_cli::{audit::audit, python::py_handshake, utils::get_startup_options, watch::watch};
use std::io::Write;
use termcolor::{ColorChoice, StandardStream};

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
            SubCommand::with_name("audit")
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
                    Arg::with_name("verbosity")
                        .short("v")
                        .help("increase the verbosity level of results")
                        .multiple(true)
                        .takes_value(false),
                )
                .arg(
                    Arg::with_name("persist")
                        .long("persist")
                        .value_name("PERSIST")
                        .help("persist the evaluation files")
                        .takes_value(false),
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
                    Arg::with_name("config")
                        .short("c")
                        .long("config")
                        .value_name("CONFIG")
                        .help("the config file to use")
                        .takes_value(true),
                ),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("audit") {
        let startup_options = get_startup_options(matches, &mut stdout).unwrap();
        audit(
            &mut stdout,
            &startup_options.filepath,
            &startup_options.config_file,
            &startup_options.result_file,
            startup_options.verbosity,
            startup_options.persist,
        );
    }

    if let Some(matches) = matches.subcommand_matches("watch") {
        if let Err(e) = writeln!(stdout, "[INFO] PySprint watch mode starting.") {
            println!("Error writing to stdout: {}", e);
        }
        let startup_options = get_startup_options(matches, &mut stdout).unwrap();

        let _ = py_handshake(&mut stdout);

        if let Err(e) = writeln!(stdout, "[INFO] Watch started..") {
            println!("Error writing to stdout: {}", e);
        }

        if let Err(e) = watch(
            &startup_options.filepath,
            &startup_options.config_file,
            startup_options.persist,
            &startup_options.result_file,
            startup_options.verbosity,
            &mut stdout,
        ) {
            if let Err(e) = writeln!(stdout, "[ERROR] error watching..: {:?}", e) {
                println!("Error writing to stdout: {}", e);
            }
        }
    }
}
