use crate::deserialize::MethodType;
use crate::{
    codegen::{render_generic_template, write_tempfile_with_imports},
    parser::parse,
    python::exec_py,
};
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use std::path::Path;
use std::time::Duration;
use std::{io, io::Write};
use termcolor::StandardStream;

pub fn watch<P: AsRef<Path> + Copy>(
    stdout: &mut StandardStream,
    path: P,
    config_file: &str,
    result_file: &str,
    verbosity: u8,
    persist: bool,
) -> notify::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher = watcher(tx, Duration::from_millis(100)).unwrap();
    watcher.watch(path, RecursiveMode::NonRecursive)?;

    // we need to append the filepath to the template, because python also runs from *here*.
    let fpath = &path.as_ref().to_str().unwrap();
    let config = parse(&format!("{}/{}", fpath, config_file)).unwrap();

    match &config.method {
        MethodType::CosFitMethod | MethodType::SPPMethod => {
            panic!("CosFitMethod and SPPMethod are not supported in watch mode.");
        }
        _ => {}
    }

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
                                if config
                                    .load_options
                                    .extensions
                                    .clone()
                                    .into_comparable()
                                    .contains(&value.to_str().unwrap().to_owned())
                                {
                                    // clear terminal on rerun
                                    print!("\x1B[2J\x1B[1;1H");
                                    // stdout is frequently line-buffered by default so it is necessary
                                    // to flush() to ensure the clear above is emitted immediately
                                    io::stdout().flush().unwrap();

                                    // render the code that needs to be executed
                                    let code = render_generic_template(
                                        e.file_name().unwrap().to_str().unwrap(),
                                        fpath,
                                        &config,
                                        result_file,
                                        verbosity,
                                        false,
                                        None,
                                        None,
                                    );

                                    // write the generated code if needed
                                    if persist {
                                        let _ = write_tempfile_with_imports(
                                            e.file_stem().unwrap().to_str().unwrap(),
                                            code.as_ref().unwrap(),
                                            fpath,
                                        );
                                    }

                                    // execute it
                                    let _ = exec_py(&code.unwrap(), stdout, false);
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
