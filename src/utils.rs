use crate::codegen::maybe_write_default_yaml;
use crate::deserialize::LoadOptions;
use crate::io::create_results_file;
use clap::ArgMatches;
use indicatif::{ProgressBar, ProgressStyle};
use std::io::Write;
use std::path::{Path, PathBuf};
use termcolor::{Color, ColorSpec, StandardStream, WriteColor};
use wildmatch::WildMatch;

#[derive(Debug, Clone)]
pub struct StartupOptions {
    pub filepath: String,
    pub config_file: String,
    pub result_file: String,
    pub verbosity: u8,
    pub persist: bool,
}

pub fn get_startup_options(
    matches: &ArgMatches<'_>,
    stdout: &mut StandardStream,
) -> Option<StartupOptions> {
    let verbosity: u8 = match matches.occurrences_of("verbosity") {
        0 => 0,
        _ => 1,
    };
    let persist = matches.is_present("persist");

    if let Some(filepath) = matches.value_of("path") {
        let config_file = matches.value_of("config").unwrap_or("eval.yaml");
        let config_filepath = Path::new(&filepath).join(config_file);
        if !config_filepath.exists() {
            let _ = stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)));
            let _ = writeln!(
                stdout,
                "[WARN] File named {:?} currently doesn't exist.",
                config_filepath
            );
            let _ = WriteColor::reset(stdout);
            maybe_write_default_yaml(filepath);
        }

        let result_file = matches.value_of("result").unwrap_or("results.json");
        let result_filepath = Path::new(&filepath).join(result_file);
        if matches.is_present("override") {
            if result_filepath.exists() {
                let _ = writeln!(
                    stdout,
                    "[INFO] Overriding result file at {:?}.",
                    &result_filepath
                );
            } else if let Err(e) = writeln!(
                stdout,
                "[INFO] Created {:?} result file.",
                result_filepath.to_str().unwrap()
            ) {
                println!("Error writing to stdout: {}", e);
            }
            create_results_file(result_filepath.into_os_string().to_str().unwrap()).unwrap();
        } else if !result_file_is_present(&result_filepath, stdout).unwrap_or(true) {
            create_results_file(result_filepath.into_os_string().to_str().unwrap()).unwrap();
        } else {
            let _ = writeln!(
                stdout,
                "[INFO] Type 'yes' or 'y' to override it, or anything else to quit.",
            );
            if maybe_override_results_file() {
                create_results_file(result_filepath.clone().into_os_string().to_str().unwrap())
                    .unwrap();
                let _ = writeln!(
                    stdout,
                    "[INFO] Result file overridden at {:?}.",
                    &result_filepath.to_str().unwrap()
                );
            } else {
                panic!("failed to find a writeable result file.");
            }
        }

        return Some(StartupOptions {
            filepath: filepath.into(),
            config_file: config_file.into(),
            result_file: result_file.into(),
            verbosity,
            persist,
        });
    }
    None
}

pub fn result_file_is_present<P: AsRef<Path>>(
    result_filepath: P,
    stdout: &mut StandardStream,
) -> Result<bool, Box<dyn std::error::Error>> {
    if result_filepath.as_ref().exists() {
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))?;
        let warning = format!(
            "[WARN] The result file named {:?} already exists.",
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

pub fn get_exclude_patterns(file_pattern_options: &LoadOptions) -> Vec<WildMatch> {
    let mut ep: Vec<WildMatch> = Vec::new();
    for pattern in &file_pattern_options
        .exclude_patterns
        .clone()
        .into_comparable()
    {
        ep.push(WildMatch::new(pattern));
    }
    ep
}

#[must_use]
pub fn get_process_bar_with_length(l: u64) -> ProgressBar {
    let bar = ProgressBar::new(l);
    bar.set_style(
        ProgressStyle::default_bar().template("{prefix:>16.green} [{bar:50}] {pos}/{len} {msg}"),
    );
    bar.set_prefix("Processing files");
    bar
}

#[must_use]
pub fn get_spinner() -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    let spinner_style = ProgressStyle::default_spinner()
        // .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
        .tick_chars("/|\\- ")
        .template("{prefix:.bold.dim} {spinner} {wide_msg}");
    pb.set_style(spinner_style);
    pb.enable_steady_tick(40);
    pb
}

pub fn sort_by_arms(
    files: &[PathBuf],
    stdout: &mut StandardStream,
    warn: bool,
) -> (Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>) {
    let mut ifgs = Vec::<PathBuf>::new();
    let mut sams = Vec::<PathBuf>::new();
    let mut refs = Vec::<PathBuf>::new();

    // exclude the hanging files, the arms missmatch somewhere
    let n = files.len() - files.len() % 3;
    if n != files.len() && warn {
        let _ = stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)));
        let _ = writeln!(
            stdout,
            "[WARN] The number of files is not divisible by 3, it's very likely that this command will fail..
       Maybe you forgot to exclude/include some?"
        );
        let _ = WriteColor::reset(stdout);
    }
    for file in files.iter().take(n).step_by(3) {
        ifgs.push(file.to_path_buf());
    }
    for file in files.iter().take(n).skip(1).step_by(3) {
        sams.push(file.to_path_buf());
    }
    for file in files.iter().take(n).skip(2).step_by(3) {
        refs.push(file.to_path_buf());
    }
    (ifgs, sams, refs)
}

#[must_use]
pub fn maybe_override_results_file() -> bool {
    let mut input_text = String::new();
    std::io::stdin()
        .read_line(&mut input_text)
        .expect("failed to read from stdin");

    matches!(input_text.to_lowercase().trim(), "yes" | "y")
}
