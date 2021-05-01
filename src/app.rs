use crate::statistics::summarize;
use crate::{audit::audit, python::py_handshake, utils::get_startup_options, watch::watch};
use clap::{
    crate_authors, crate_description, crate_version, App, AppSettings, Arg, ArgMatches, SubCommand,
};
use std::io::Write;
use termcolor::{ColorChoice, StandardStream};

pub fn launch() {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    let matches = start_app_and_get_matches();

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
            &mut stdout,
            &startup_options.filepath,
            &startup_options.config_file,
            &startup_options.result_file,
            startup_options.verbosity,
            startup_options.persist,
        ) {
            if let Err(e) = writeln!(stdout, "[ERROR] error watching..: {:?}", e) {
                println!("Error writing to stdout: {}", e);
            }
        }
    }

    if let Some(matches) = matches.subcommand_matches("summarize") {
        let result_file = matches.value_of("result").unwrap_or("results.json");
        summarize(result_file);
    }
}

fn start_app_and_get_matches() -> ArgMatches<'static> {
    App::new("PySprint-CLI")
        .setting(AppSettings::ColorAlways)
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .subcommand(
            SubCommand::with_name("watch")
                .about("Watch a directory for changes, immediately rerun on events.")
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
                        .short("p")
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
                )
                .arg(
                    Arg::with_name("override")
                        .long("override")
                        .short("o")
                        .help("whether to override existing result file")
                        .takes_value(false),
                ),
        )
        .subcommand(
            SubCommand::with_name("audit")
                .about("Evaluate a whole directory of files.")
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
                        .short("p")
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
                )
                .arg(
                    Arg::with_name("override")
                        .long("override")
                        .short("o")
                        .help("whether to override existing result file")
                        .takes_value(false),
                ),
        )
        .subcommand(
            SubCommand::with_name("summarize")
                .about("Summarize the results.")
                .arg(
                    Arg::with_name("result")
                        .short("r")
                        .long("result")
                        .value_name("RESULT")
                        .help("the result file to summarize")
                        .takes_value(true),
                ),
        )
        .get_matches()
}

#[test]
fn app_is_valid() {
    launch();
}
