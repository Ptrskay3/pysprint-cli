use crate::parser::FilePatternOptions;
use crate::utils::get_exclude_patterns;
use std::fs::File;
use std::io::Write;
use std::{ffi::OsStr, fs, io, path::PathBuf};

pub fn create_results_file(filename: &str) -> std::io::Result<()> {
    let mut file = File::create(filename)?;
    file.write_all(b"{ }")?;
    Ok(())
}

pub fn get_files(
    root: &str,
    file_pattern_options: &FilePatternOptions,
) -> io::Result<Vec<PathBuf>> {
    let mut result = vec![];

    // TODO: needless collect

    // Vec<String> -> Vec<&str> conversion, to be comparable below
    let ext_as_str_ref = file_pattern_options
        .extensions
        .iter()
        .map(|s| &s[..])
        .collect::<Vec<&str>>();

    let skips_as_str_ref = file_pattern_options
        .skip_files
        .iter()
        .map(|s| &s[..])
        .collect::<Vec<&str>>();

    for path in fs::read_dir(root)? {
        let path = path?.path();

        // skip directories, we dont walk recursively at the moment
        if path.is_dir() {
            continue;
        }
        // early bailout of skip files
        if skips_as_str_ref.contains(
            &path
                .file_name()
                .and_then(OsStr::to_str)
                .unwrap_or("__nofilename"),
        ) {
            continue;
        }
        // pick up files that have the specified extensions
        if ext_as_str_ref.contains(
            &path
                .extension()
                .and_then(OsStr::to_str)
                .unwrap_or("__noextension"),
        ) {
            result.push(path.to_owned());
        }
    }

    let exclude_patterns = get_exclude_patterns(file_pattern_options);

    // exclude every file that matches any pre-defined pattern
    result.retain(|path| {
        !exclude_patterns
            .iter()
            .map(|pattern| pattern.matches(&path.to_str().unwrap()))
            .any(|op| op)
    });

    result.sort();
    Ok(result)
}