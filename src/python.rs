use indicatif::{ProgressBar, ProgressStyle};
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use std::io::Write;
use termcolor::{Color, ColorSpec, StandardStream, WriteColor};

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
    if exec_py("True", stdout).is_err() {
        panic!("Python interpreter crashed..")
    }

    pb.finish_and_clear();
}

/// Execute Python code passed as &str.
pub fn exec_py(content: &str, stdout: &mut StandardStream) -> PyResult<()> {
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
        py.check_signals()?;
        let _ = WriteColor::reset(stdout);
    }
    Ok(())
}
