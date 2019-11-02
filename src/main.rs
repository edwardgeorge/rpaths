use std::fs::{read_dir, read_link, DirEntry, File};
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
use std::vec::Vec;

use clap::{App, Arg};
use expanduser::expanduser;

fn is_symlink(path: &Path) -> io::Result<bool> {
    let ft = path.symlink_metadata()?.file_type();
    Ok(ft.is_symlink())
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

fn make_canonical(dir: &Path, path: PathBuf) -> Option<PathBuf> {
    if path.is_absolute() {
        if path.exists() {
            Some(path)
        } else {
            None
        }
    } else {
        match dir.join(path).canonicalize() {
            Ok(p) => Some(p),
            Err(_) => None,
        }
    }
}

fn dir_paths(path: &Path) -> io::Result<Vec<String>> {
    let entries = dir_entries(path);
    let mut paths = Vec::new();
    for entry in entries {
        let p = entry.path();
        if is_symlink(&p)? {
            match read_link(entry.path())
                .ok()
                .and_then(|x| make_canonical(path, x))
            {
                Some(p) => paths.push(p.to_string_lossy().into_owned()),
                None => (),
            }
        } else if p.is_file() {
            paths.append(&mut file_paths(&p));
        } else {
        }
    }
    Ok(paths)
}

fn file_paths(path: &Path) -> Vec<String> {
    File::open(path)
        .and_then(|file| {
            let mut entries = Vec::new();
            let reader = io::BufReader::new(file);
            for line in reader.lines() {
                let line = line?;
                entries.push(line);
            }
            Ok(entries)
        })
        .unwrap_or_else(|_| vec![])
}

fn find_paths(include_sys: bool) -> io::Result<Vec<String>> {
    let path = expanduser("~/.paths.d")?;
    let result = vec![
        dir_paths(&path)?,
        if include_sys {
            dir_paths(Path::new("/etc/paths.d"))?
        } else {
            vec![]
        },
        if include_sys {
            file_paths(Path::new("/etc/paths"))
        } else {
            vec![]
        },
    ]
    .into_iter()
    .flatten()
    .collect();
    Ok(result)
}

fn main() -> io::Result<()> {
    let matches = App::new("rpaths")
        .arg(
            Arg::with_name("system")
                .short("s")
                .long("system")
                .help(
                    "includes system paths. emulates behaviour of OSX path_helper, appending paths",
                )
                .takes_value(false)
                .required(false)
                .multiple(false),
        )
        .get_matches();
    let sys = matches.is_present("system");
    let res = find_paths(sys)?;
    println!("{}", res.join(":"));
    Ok(())
}
