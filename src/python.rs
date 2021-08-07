use crate::utils::get_spinner;
use pyo3::ffi::Py_SetPythonHome;
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use std::io::Write;
use std::path::PathBuf;
use termcolor::{Color, ColorSpec, StandardStream, WriteColor};

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
    
    // A workaround for https://github.com/ContinuumIO/anaconda-issues/issues/11439
    // by https://github.com/cgranade

    // Due to https://github.com/ContinuumIO/anaconda-issues/issues/11439,
    // we first need to set PYTHONHOME. To do so, we will look for whatever
    // directory on PATH currently has python.exe.
    let python_exe = which::which("python").expect("Python was not found on PATH.");
    let python_home = python_exe.parent().unwrap();

    // The Python C API uses null-terminated UTF-16 strings, so we need to
    // encode the path into that format here.
    // We could use the Windows FFI modules provided in the standard library,
    // but we want this to work cross-platform, so we do things more manually.
    let mut python_home = python_home
        .to_str()
        .unwrap()
        .encode_utf16()
        .collect::<Vec<u16>>();
    python_home.push(0);
    unsafe {
        Py_SetPythonHome(python_home.as_ptr());
    }

    // Once we've set the configuration we need, we can go on and manually
    // initialize PyO3.
    pyo3::prepare_freethreaded_python();

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

    let result = py.run(content, None, Some(locals));

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
    
    // A workaround for https://github.com/ContinuumIO/anaconda-issues/issues/11439
    // by https://github.com/cgranade

    // Due to https://github.com/ContinuumIO/anaconda-issues/issues/11439,
    // we first need to set PYTHONHOME. To do so, we will look for whatever
    // directory on PATH currently has python.exe.
    let python_exe = which::which("python").expect("Python was not found on PATH.");
    let python_home = python_exe.parent().unwrap();

    // The Python C API uses null-terminated UTF-16 strings, so we need to
    // encode the path into that format here.
    // We could use the Windows FFI modules provided in the standard library,
    // but we want this to work cross-platform, so we do things more manually.
    let mut python_home = python_home
        .to_str()
        .unwrap()
        .encode_utf16()
        .collect::<Vec<u16>>();
    python_home.push(0);
    unsafe {
        Py_SetPythonHome(python_home.as_ptr() as *const i32);
    }

    // Once we've set the configuration we need, we can go on and manually
    // initialize PyO3.
    pyo3::prepare_freethreaded_python();

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
