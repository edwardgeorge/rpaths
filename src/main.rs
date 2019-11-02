use std::fs::{read_dir, read_link, DirEntry};
use std::io;
use std::path::{Path, PathBuf};
use std::vec::Vec;

use expanduser::expanduser;

fn combine_paths(inp: &Vec<PathBuf>, sep: &str) -> String {
    let x: Vec<String> = inp
        .iter()
        .map(|x| x.to_string_lossy().into_owned())
        .collect();
    x[..].join(sep)
}

fn dir_entries(path: &Path) -> Vec<DirEntry> {
    read_dir(path)
        .and_then(|entries| {
            let mut x: Vec<_> = entries.flatten().collect();
            x.sort_unstable_by(|a, b| a.path().cmp(&b.path()));
            Ok(x)
        })
        .unwrap_or_else(|_| vec![])
}

fn dir_links(path: &Path) -> Vec<PathBuf> {
    let entries = dir_entries(path);
    let mut paths = Vec::new();
    for entry in entries {
        match read_link(entry.path()).and_then(|x| path.join(x).canonicalize()) {
            Ok(p) => paths.push(p),
            Err(_) => (),
        }
    }
    return paths;
}

fn main() -> io::Result<()> {
    let path = expanduser("~/.paths.d")?;
    let paths = dir_links(path.as_path());
    let res = combine_paths(&paths, ":");
    println!("{}", res);
    Ok(())
}
