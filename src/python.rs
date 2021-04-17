use crate::utils::get_spinner;
use pyo3::ffi;
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use std::io::Write;
use std::path::PathBuf;
use termcolor::{Color, ColorSpec, StandardStream, WriteColor};
use widestring::WideCString;

/// Check if we're able to start a Python interpreter,
/// and fail early if we can't.
pub fn py_handshake(stdout: &mut StandardStream) {
    let pb = get_spinner();
    pb.set_message("Waking up the Python interpreter..");

    // A quick check whether Python is ready.
    if exec_py("True", stdout, false).is_err() {
        panic!("Python interpreter crashed.. Do you have pysprint installed?")
    }

    pb.finish_and_clear();
}

#[cfg(target_os = "windows")]
pub fn exec_py(
    content: &str,
    stdout: &mut StandardStream,
    to_file: bool,
) -> PyResult<(bool, String)> {
    // if there is `CONDA_PREFIX`, set PYTHONHOME to the same thing.
    // related issue: https://github.com/ContinuumIO/anaconda-issues/issues/11439
    // this is potentially unsafe and not tested, so it should be avoided if the issue gets resolved
    if let Some(python_home) = std::env::var_os("CONDA_PREFIX") {
        unsafe {
            ffi::Py_SetPythonHome(
                WideCString::from_str(python_home.to_str().unwrap())
                    .unwrap()
                    .as_ptr(),
            );
        }
    }

    // whether this run resulted in an error
    // we count the fails in audit using this variable
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

#[cfg(not(target_os = "windows"))]
pub fn exec_py(
    content: &str,
    stdout: &mut StandardStream,
    to_file: bool,
) -> PyResult<(bool, String)> {
    // if there is `CONDA_PREFIX`, set PYTHONHOME to the same thing.
    // related issue: https://github.com/ContinuumIO/anaconda-issues/issues/11439
    // this is potentially unsafe and not tested, so it should be avoided if the issue gets resolved
    if let Some(python_home) = std::env::var_os("CONDA_PREFIX") {
        unsafe {
            ffi::Py_SetPythonHome(
                WideCString::from_str(python_home.to_str().unwrap())
                    .unwrap()
                    .as_ptr() as *const i32,
            );
        }
    }

    // whether this run resulted in an error
    // we count the fails in audit using this variable
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
