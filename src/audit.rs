use crate::codegen::{render_generic_template, render_spp_template, write_tempfile_with_imports};
use crate::io::get_files;
use crate::parser::parse;
use crate::python::{exec_py, py_handshake, write_err};
use crate::utils::{get_process_bar_with_length, get_spinner, sort_by_arms};
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
    let (mut evaluate_options, intermediate_hooks, file_pattern_options) =
        parse(&format!("{}/{}", filepath, config_file));

    let _ = py_handshake(stdout);

    let files = get_files(filepath, &file_pattern_options).unwrap();
    match evaluate_options.text_options["methodname"].as_ref() {
        "SPPMethod" => {
            let (mut ifgs, mut sams, mut refs) = sort_by_arms(&files, stdout);
            let modulo = evaluate_options
                .number_options
                .entry("mod".into())
                .or_insert_with(|| Box::new(1.0));
            match **modulo as i32 {
                3 => {}
                1 => {
                    ifgs = files;
                    sams = vec![];
                    refs = vec![];
                }
                -1 => {
                    sams = vec![];
                    refs = vec![];
                }
                _ => {
                    panic!("mod field should be 3, 1 or -1, found {}", modulo);
                }
            };

            let code = render_spp_template(
                &ifgs,
                &refs,
                &sams,
                filepath,
                &evaluate_options,
                &intermediate_hooks,
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
        "CosFitMethod" => {
            let (ifgs, sams, refs) = sort_by_arms(&files, stdout);

            // TODO: grouping

            let bar = get_process_bar_with_length(ifgs.len() as u64);

            for (idx, file) in ifgs.iter().enumerate() {
                bar.inc(1);
                let code = render_generic_template(
                    file.as_path().file_name().unwrap().to_str().unwrap(),
                    filepath,
                    &evaluate_options,
                    &intermediate_hooks,
                    &result_file,
                    verbosity,
                    true,
                    Some((&sams[idx], &refs[idx])),
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
                pb.finish_with_message(&format!("Report generated at `{}/errors.log`.", filepath));
            }
        }
        _ => {
            let bar = get_process_bar_with_length(files.len() as u64);

            for file in files.iter() {
                bar.inc(1);

                // render the code that needs to be executed
                let code = render_generic_template(
                    file.as_path().file_name().unwrap().to_str().unwrap(),
                    filepath,
                    &evaluate_options,
                    &intermediate_hooks,
                    &result_file,
                    verbosity,
                    true,
                    None,
                );

                // write the generated code if needed
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
                pb.finish_with_message(&format!("Report generated at `{}/errors.log`.", filepath));
            }
        }
    }
}
