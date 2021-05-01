use crate::codegen::{render_generic_template, render_spp_template, write_tempfile_with_imports};
use crate::deserialize::{MethodType, _Mod};
use crate::io::get_files;
use crate::parser::parse;
use crate::python::{exec_py, py_handshake, write_err};
use crate::utils::{get_process_bar_with_length, get_spinner, sort_by_arms};
use itertools::izip;
use std::io::Write;
use termcolor::{Color, ColorSpec, StandardStream, WriteColor};

pub fn audit(
    stdout: &mut StandardStream,
    filepath: &str,
    config_file: &str,
    result_file: &str,
    verbosity: u8,
    persist: bool,
) {
    let mut counter = 0;
    let mut traceback = String::new();
    let config = parse(&format!("{}/{}", filepath, config_file)).unwrap();

    let _ = py_handshake(stdout);

    let files = get_files(filepath, &config.load_options).unwrap();
    let warn = config.method == MethodType::SPPMethod
        || config.method == MethodType::CosFitMethod
        || config.method == MethodType::MinMaxMethod;
    let (mut ifgs, mut sams, mut refs) = sort_by_arms(&files, stdout, warn);
    let modulo = config.load_options._mod;
    match &config.method {
        MethodType::SPPMethod => {
            match modulo.unwrap() {
                _Mod(3) => {}
                _Mod(1) => {
                    ifgs = files;
                    sams = vec![];
                    refs = vec![];
                }
                _Mod(-1) => {
                    sams = vec![];
                    refs = vec![];
                }
                _ => {
                    panic!("mod field should be 3, 1 or -1, found {:?}", modulo);
                }
            };

            let code = render_spp_template(
                &ifgs,
                &refs,
                &sams,
                filepath,
                &config,
                &result_file,
                verbosity,
                true,
            );

            if persist {
                let _ = write_tempfile_with_imports("spp_eval", code.as_ref().unwrap(), filepath);
            }

            if let Err(e) = exec_py(&code.unwrap(), stdout, false) {
                let _ = stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)));
                let py_error = format!("[ERRO] Python error:\n{:?}", e);
                if let Err(e) = writeln!(stdout, "{}", py_error) {
                    println!("Error writing to stdout: {}", e);
                }
                let _ = WriteColor::reset(stdout);
            }
        }
        _ => {
            // TODO: this is a really ugly, almost copy-paste match statement
            // there must be a way to solve this elegantly..
            match modulo.unwrap() {
                _Mod(3) => {
                    let bar = get_process_bar_with_length(ifgs.len() as u64);

                    for (file, sam_, ref_) in izip!(&ifgs, &sams, &refs) {
                        bar.inc(1);
                        let code = render_generic_template(
                            file.as_path().file_name().unwrap().to_str().unwrap(),
                            filepath,
                            &config,
                            &result_file,
                            verbosity,
                            true,
                            Some(&sam_),
                            Some(&ref_),
                        );
                        if persist {
                            let _ = write_tempfile_with_imports(
                                file.as_path().file_stem().unwrap().to_str().unwrap(),
                                code.as_ref().unwrap(),
                                filepath,
                            );
                        }
                        // execute it
                        if let Ok((e, tb)) = exec_py(&code.unwrap(), stdout, true) {
                            if e {
                                counter += 1;
                                traceback.push_str(&format!(
                                    "file: {}\terror: {}\n",
                                    file.as_path()
                                        .file_name()
                                        .unwrap()
                                        .to_str()
                                        .unwrap_or("unknown filename"),
                                    &tb
                                ));
                            }
                        }
                    }
                    bar.finish_with_message("Done.");
                    if counter > 0 {
                        if let Err(e) =
                            writeln!(stdout, "[INFO] {:?} files skipped or errored out.", counter)
                        {
                            println!("Error writing to stdout: {:?}", e);
                        }
                        let pb = get_spinner();
                        pb.set_message("Generating report..");
                        let _ = write_err(filepath, &traceback);
                        pb.finish_with_message(&format!(
                            "Report generated at `{}/errors.log`.",
                            filepath
                        ));
                    }
                }
                _Mod(1) => {
                    let bar = get_process_bar_with_length(files.len() as u64);

                    for file in &files {
                        bar.inc(1);
                        let code = render_generic_template(
                            file.as_path().file_name().unwrap().to_str().unwrap(),
                            filepath,
                            &config,
                            &result_file,
                            verbosity,
                            true,
                            None,
                            None,
                        );
                        if persist {
                            let _ = write_tempfile_with_imports(
                                file.as_path().file_stem().unwrap().to_str().unwrap(),
                                code.as_ref().unwrap(),
                                filepath,
                            );
                        }
                        // execute it
                        if let Ok((e, tb)) = exec_py(&code.unwrap(), stdout, true) {
                            if e {
                                counter += 1;
                                traceback.push_str(&format!(
                                    "file: {}\terror: {}\n",
                                    file.as_path()
                                        .file_name()
                                        .unwrap()
                                        .to_str()
                                        .unwrap_or("unknown filename"),
                                    &tb
                                ));
                            }
                        }
                    }
                    bar.finish_with_message("Done.");
                    if counter > 0 {
                        if let Err(e) =
                            writeln!(stdout, "[INFO] {:?} files skipped or errored out.", counter)
                        {
                            println!("Error writing to stdout: {:?}", e);
                        }
                        let pb = get_spinner();
                        pb.set_message("Generating report..");
                        let _ = write_err(filepath, &traceback);
                        pb.finish_with_message(&format!(
                            "Report generated at `{}/errors.log`.",
                            filepath
                        ));
                    }
                }
                _Mod(-1) => {
                    let bar = get_process_bar_with_length(ifgs.len() as u64);

                    for file in &ifgs {
                        bar.inc(1);
                        let code = render_generic_template(
                            file.as_path().file_name().unwrap().to_str().unwrap(),
                            filepath,
                            &config,
                            &result_file,
                            verbosity,
                            true,
                            None,
                            None,
                        );
                        if persist {
                            let _ = write_tempfile_with_imports(
                                file.as_path().file_stem().unwrap().to_str().unwrap(),
                                code.as_ref().unwrap(),
                                filepath,
                            );
                        }
                        // execute it
                        if let Ok((e, tb)) = exec_py(&code.unwrap(), stdout, true) {
                            if e {
                                counter += 1;
                                traceback.push_str(&format!(
                                    "file: {}\terror: {}\n",
                                    file.as_path()
                                        .file_name()
                                        .unwrap()
                                        .to_str()
                                        .unwrap_or("unknown filename"),
                                    &tb
                                ));
                            }
                        }
                    }
                    bar.finish_with_message("Done.");
                    if counter > 0 {
                        if let Err(e) =
                            writeln!(stdout, "[INFO] {:?} files skipped or errored out.", counter)
                        {
                            println!("Error writing to stdout: {:?}", e);
                        }
                        let pb = get_spinner();
                        pb.set_message("Generating report..");
                        let _ = write_err(filepath, &traceback);
                        pb.finish_with_message(&format!(
                            "Report generated at `{}/errors.log`.",
                            filepath
                        ));
                    }
                }
                _ => {
                    panic!("mod field should be 3, 1 or -1, found {:?}", modulo);
                }
            };
        }
    }
}
