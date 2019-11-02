use std::fs::{read_dir, read_link, DirEntry, File};
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
use std::vec::Vec;

use clap::{App, Arg};
use expanduser::expanduser;

fn pathbufs_to_strings(inp: &Vec<PathBuf>, sep: &str) -> Vec<String> {
    inp.iter()
        .map(|x| x.to_string_lossy().into_owned())
        .collect()
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

fn make_canonical<'a>(dir: &Path, path: PathBuf) -> Option<PathBuf> {
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

fn dir_links(path: &Path) -> Vec<PathBuf> {
    let entries = dir_entries(path);
    let mut paths = Vec::new();
    for entry in entries {
        match read_link(entry.path())
            .ok()
            .and_then(|x| make_canonical(path, x))
        {
            Some(p) => paths.push(p),
            None => (),
        }
    }
    return paths;
}

fn dir_file_entries(path: &Path) -> Vec<String> {
    let entries = dir_entries(path);
    let mut paths = Vec::new();
    for entry in entries {
        let p = entry.path();
        let mut fentries = file_entries(&p);
        paths.append(&mut fentries);
    }
    return paths;
}

fn file_entries(path: &Path) -> Vec<String> {
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
    let mut syspaths = if matches.is_present("system") {
        let mut r = file_entries(Path::new("/etc/paths"));
        r.append(&mut dir_file_entries(Path::new("/etc/paths.d")));
        r
    } else {
        vec![]
    };
    let path = expanduser("~/.paths.d")?;
    let paths = dir_links(path.as_path());
    let mut res = pathbufs_to_strings(&paths, ":");
    res.append(&mut syspaths);
    // let r = vec![res, foo]
    //     .into_iter()
    //     .flatten()
    //     .collect::<Vec<_>>()
    //     .join(":");
    println!("{}", res.join(":"));
    Ok(())
}
