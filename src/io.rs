use std::collections::HashSet;
use std::{
    fs::File,
    io::{self, BufRead},
    path::PathBuf,
};

pub fn load_proxies(path: Option<PathBuf>) -> Option<Vec<String>> {
    path.map(|p| {
        File::open(p)
            .map(io::BufReader::new)
            .map(|reader| reader.lines().map_while(Result::ok).collect::<Vec<_>>())
            .unwrap_or_default()
    })
}

pub fn load_urls(path: &PathBuf) -> io::Result<Vec<String>> {
    let reader = io::BufReader::new(File::open(path)?);
    let mut seen = HashSet::new();

    Ok(reader
        .lines()
        .map_while(Result::ok)
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
        .filter(|s| !s.starts_with('#'))
        .filter(|s| seen.insert(s.clone()))
        .collect())
}
