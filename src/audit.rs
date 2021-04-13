use crate::parser::FilePatternOptions;
use std::{ffi::OsStr, fs, io, io::Write, path::PathBuf};
use termcolor::{Color, ColorSpec, StandardStream, WriteColor};
use wildmatch::WildMatch;

pub fn sort_by_arms(
    files: &[PathBuf],
    stdout: &mut StandardStream,
) -> (Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>) {
    let mut ifgs = Vec::<PathBuf>::new();
    let mut sams = Vec::<PathBuf>::new();
    let mut refs = Vec::<PathBuf>::new();

    // exclude the hanging files, the arms missmatch somewhere
    let n = files.len() - files.len() % 3;
    if n != files.len() {
        let _ = stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)));
        let _ = writeln!(
            stdout,
            "[WARN] The number of files is not divisible by 3..
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

fn get_exclude_patterns(file_pattern_options: &FilePatternOptions) -> Vec<WildMatch> {
    let mut exclude_patterns: Vec<WildMatch> = Vec::new();
    for pattern in &file_pattern_options.exclude_patterns {
        exclude_patterns.push(WildMatch::new(&pattern));
    }
    exclude_patterns
}
