use crate::Result;
use std::path::{Path, PathBuf};

pub fn list_dirs(input: &str) -> Vec<&Path> {
    if input.is_empty() {
        return vec![];
    }

    input
        .split(':')
        .filter_map(|p| {
            let path = Path::new(p);
            if path.is_dir() {
                Some(path)
            } else {
                None
            }
        })
        .collect()
}

pub fn list_files(path: &Path) -> Result<Vec<PathBuf>> {
    let mut paths: Vec<PathBuf> = vec![];
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            paths.push(path);
        }
    }
    Ok(paths)
}

pub fn filename(name: &str) -> Box<dyn Fn(&PathBuf) -> bool> {
    let name = name.to_owned();
    Box::new(move |path| path.file_name().is_some_and(|n| n == name.as_str()))
}

pub fn current_dir() -> Result<PathBuf> {
    let dir = std::path::absolute(".")?;
    Ok(dir)
}

pub fn path_stringify(path: PathBuf) -> Option<String> {
    path.to_str().map(|p| p.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_split_into_directories() {
        let input = "";
        let dirs = list_dirs(input);
        assert!(dirs.is_empty());
    }
}
