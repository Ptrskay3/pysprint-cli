use indicatif::{ProgressBar, ProgressStyle};
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use pyo3::ffi;
use std::io::Write;
use std::path::PathBuf;
use termcolor::{Color, ColorSpec, StandardStream, WriteColor};
use widestring::WideCString;


/// Check if we're able to start a Python interpreter,
/// and fail early if we can't.
pub fn py_handshake(stdout: &mut StandardStream) {
    // setup the spinner
    let pb = ProgressBar::new_spinner();
    let spinner_style = ProgressStyle::default_spinner()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
        .template("{prefix:.bold.dim} {spinner} {wide_msg}");
    pb.set_style(spinner_style);
    pb.set_message("Waking up the Python interpreter..");
    pb.enable_steady_tick(40);

    // A quick check whether Python is ready.
    if exec_py("True", stdout, false).is_err() {
        panic!("Python interpreter crashed..")
    }

    pb.finish_and_clear();
}

/// Execute Python code passed as &str.
pub fn exec_py(
    content: &str,
    stdout: &mut StandardStream,
    to_file: bool,
) -> PyResult<(bool, String)> {

    // if there is `CONDA_PREFIX`, set PYTHONHOME
    if let Some(PYTHONHOME) = std::env::var_os("CONDA_PREFIX") {
            unsafe {
                ffi::Py_SetPythonHome(
                    WideCString::from_str(PYTHONHOME.to_str().unwrap())
                        .unwrap()
                        .as_ptr(),
                );
            }
        }

    let mut is_err = false;

    // the error, if exists..
    let mut traceback = String::new();

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
        // TODO: write error to file.
        is_err = true;
        traceback = err.to_string();
        if !to_file {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
            let py_error = format!("[ERRO] Python error:\n{:?}", err);
            if let Err(e) = writeln!(stdout, "{}", py_error) {
                println!("Error writing to stdout: {}", e);
            }
            py.check_signals()?;
            let _ = WriteColor::reset(stdout);
        }
    }
    Ok((is_err, traceback))
}

pub fn write_err(path: &str, content: &str) -> std::io::Result<()> {
    let cfg_path = PathBuf::from(path).join("errors.log");
    std::fs::write(cfg_path, content.as_bytes())?;
    Ok(())
}
