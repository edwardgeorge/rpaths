use std::fs::{read_dir, read_link, DirEntry, File};
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
use std::vec::Vec;

use clap::{arg, command, Arg};
use expanduser::expanduser;

const ENV_PATH: &str = "RPATHS_DIR";
const ENV_LOG: &str = "RPATHS_LOG";

fn is_symlink<P: AsRef<Path>>(path: P) -> io::Result<bool> {
    let ft = path.as_ref().symlink_metadata()?.file_type();
    Ok(ft.is_symlink())
}

fn dir_entries<P: AsRef<Path>>(path: P) -> Vec<DirEntry> {
    read_dir(path)
        .map(|entries| {
            let mut x: Vec<_> = entries.flatten().collect();
            x.sort_unstable_by_key(|a| a.path());
            x
        })
        .unwrap_or_default()
}

fn make_canonical<P: AsRef<Path>>(dir: P, path: PathBuf) -> Option<PathBuf> {
    if path.is_absolute() {
        if path.exists() {
            Some(path)
        } else {
            None
        }
    } else {
        match dir.as_ref().join(path).canonicalize() {
            Ok(p) => Some(p),
            Err(_) => None,
        }
    }
}

fn dir_paths<P: AsRef<Path>>(path: P) -> io::Result<Vec<String>> {
    log::info!("Processing directory: {}", path.as_ref().display());
    let entries = dir_entries(path.as_ref());
    let mut paths = Vec::new();
    for entry in entries {
        let p = entry.path();
        log::info!("Found entry: {}", p.display());
        if is_symlink(&p)? {
            if let Some(p2) = read_link(entry.path())
                .ok()
                .and_then(|x| make_canonical(&path, x))
            {
                log::info!("+ {} is symlink to: {}", p.display(), p2.display());
                paths.push(p2.to_string_lossy().into_owned())
            }
        } else if p.is_file() {
            log::info!("+ {} is standard file!", p.display());
            paths.append(&mut file_paths(&p));
        } else {
            log::info!("- ignoring {}", p.display());
        }
    }
    Ok(paths)
}

fn file_paths<P: AsRef<Path>>(path: P) -> Vec<String> {
    log::info!("Looking in file {} for paths...", path.as_ref().display());
    File::open(path)
        .and_then(|file| {
            let mut entries = Vec::new();
            let reader = io::BufReader::new(file);
            for line in reader.lines() {
                let line = line?;
                if Path::new(&line).exists() {
                    log::info!("Found entry: {}", line);
                    entries.push(line);
                }
            }
            Ok(entries)
        })
        .unwrap_or_default()
}

fn process_path<P: AsRef<str>>(path: P) -> io::Result<Vec<String>> {
    dir_paths(expanduser(path.as_ref())?)
}

fn find_paths<S: AsRef<str>>(
    include_default: bool,
    include_sys: bool,
    user_paths: &[S],
) -> io::Result<Vec<String>> {
    let mut res = Vec::new();
    for up in user_paths {
        res.extend(process_path(up)?);
    }
    if include_default {
        res.extend(process_path("~/.paths.d")?);
    }
    if include_sys {
        res.extend(dir_paths("/etc/paths.d")?);
        res.extend(file_paths("/etc/paths"));
    }
    Ok(res)
}

fn main() {
    env_logger::Builder::from_env(ENV_LOG).init();
    let matches = command!()
        .arg(
            arg!(system: -s --system "includes system paths. emulates behaviour of OSX path_helper, appending paths")
        )
        .arg(
            arg!(-n --"no-default")
            .requires("paths-dirs")
        )
        .arg(
            arg!(-e --"use-env")
        )
        .arg(
            Arg::new("paths-dirs")
                .num_args(1..)
                .required(false),
        )
        .get_matches();
    let sys = matches.get_flag("system");
    let mut paths: Vec<String> = matches
        .get_many("paths-dirs")
        .unwrap_or_default()
        .cloned()
        .collect();
    let no_default = matches.get_flag("no-default") || matches.get_flag("use-env");
    if matches.get_flag("use-env") {
        match std::env::var("RPATHS_DIR") {
            Ok(val) => paths.extend(val.split(':').map(|v| v.to_owned())),
            Err(err) => {
                eprintln!("Could not read {} environment variable: {}", ENV_PATH, err);
                std::process::exit(1);
            }
        }
    }
    let res = find_paths(!no_default, sys, &paths);
    match res {
        Ok(paths) => print!("{}", paths.join(":")),
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    }
}
