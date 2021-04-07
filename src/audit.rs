use crate::parser::FilePatternOptions;
use std::{ffi::OsStr, fs, io, path::PathBuf};
use wildmatch::WildMatch;

pub fn get_files(
    root: &str,
    file_pattern_options: &FilePatternOptions,
) -> io::Result<Vec<PathBuf>> {
    let mut result = vec![];

    for path in fs::read_dir(root)? {
        let path = path?.path();
        if let Some("trt") = path.extension().and_then(OsStr::to_str) {
            result.push(path.to_owned());
        }
    }

    let skips = get_exclude_patterns(file_pattern_options);

    // exclude every file that matches any pre-defined pattern
    result.retain(|path| {
        !skips
            .iter()
            .map(|pattern| pattern.matches(&path.to_str().unwrap()))
            .any(|op| op)
    });

    Ok(result)
}

fn get_exclude_patterns(file_pattern_options: &FilePatternOptions) -> Vec<WildMatch> {
    let mut exclude_patterns: Vec<WildMatch> = Vec::new();
    for pattern in &file_pattern_options.exclude_patterns {
        exclude_patterns.push(WildMatch::new(&pattern));
    }
    exclude_patterns
}
