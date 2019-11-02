use std::{io};
use std::fs::{read_dir, read_link};
use std::path::{Path, PathBuf};
use std::vec::{Vec};

use expanduser::expanduser;

fn combine_paths(inp: &Vec<PathBuf>, sep: &str) -> String {
    let x: Vec<String> = inp.iter().map(|x| x.to_string_lossy().into_owned()).collect();
    x[..].join(sep)
}

fn dir_links(dir: &Path) -> Vec<PathBuf> {
    let path = Path::new(dir);
    read_dir(path).and_then(|entries| {
        let mut paths = Vec::new();
        for entry in entries {
            let entry = entry?;
            match read_link(entry.path()).and_then(|x| path.join(x).canonicalize()) {
                Ok(p) => paths.push(p),
                Err(_) => (),
            }
        };
        Ok(paths)
    }).unwrap_or_else(|_| vec![])
}

fn main() -> io::Result<()> {
    let path = expanduser("~/.paths.d")?;
    let paths = dir_links(path.as_path());
    let res = combine_paths(&paths, ":");
    println!("{}", res);
    Ok(())
}
