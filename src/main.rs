mod cli {
    use clap::{App, Arg, SubCommand};

    pub fn make() -> App<'static, 'static> {
        App::new(env!("CARGO_PKG_NAME"))
            .version(env!("CARGO_PKG_VERSION"))
            .author(env!("CARGO_PKG_AUTHORS"))
            .about("Easily jump to projects.")
            .subcommand(
                SubCommand::with_name("add")
                    .alias("a")
                    .about("add a project alias")
                    .arg(
                        Arg::with_name("name")
                            .required(true)
                            .takes_value(true)
                            .value_name("NAME")
                    )
                    .arg(
                        Arg::with_name("dir")
                            .required(true)
                            .takes_value(true)
                            .value_name("DIRECTORY")
                    )
            )
            .subcommand(
                SubCommand::with_name("go")
                    .alias("g")
                    .about("cd to a project dir")
                    .arg(
                        Arg::with_name("name")
                            .required(true)
                            .takes_value(true)
                            .value_name("PROJECT")
                    )
            )
            .subcommand(
                SubCommand::with_name("list")
                    .alias("ls")
                    .about("list projects matching NAMEs [default: all]")
                    .arg(
                        Arg::with_name("name")
                            .multiple(true)
                            .takes_value(true)
                            .value_name("NAME")
                    )
            )
            .subcommand(
                SubCommand::with_name("open")
                    .alias("o")
                    .about("open a project dir in vscode")
                    .arg(
                        Arg::with_name("name")
                            .required(true)
                            .takes_value(true)
                            .value_name("PROJECT")
                    )
            )
            .subcommand(
                SubCommand::with_name("remove")
                    .alias("rm")
                    .about("remove a project alias")
                    .arg(
                        Arg::with_name("name")
                            .required(true)
                            .multiple(true)
                            .takes_value(true)
                            .value_name("PROJECT")
                    )
            )
    }
}

mod config {
    use serde::{Serialize, Deserialize};
    use std::path;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Project {
        pub dir: path::PathBuf,
        pub name: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Config {
        pub projects: Vec<Project>,
    }

    impl Config {
        pub fn new() -> Self {
            Config {
                projects: Vec::new(),
            }
        }
    }

    pub mod config_file {
        use std::env;
        use std::fs;
        use std::io::{self, Write};
        use std::path;
        use super::Config;

        pub fn path() -> path::PathBuf {
            let env_key = format!("{}_CONFIG_FILE", env!("CARGO_PKG_NAME"));
            let filename = env::var_os(&env_key).unwrap();

            let mut path = dirs::home_dir().expect("unable to get home dir");
            path.push(filename);

            path
        }

        pub fn contents() -> Result<Config, io::Error> {
            let file = fs::File::open(self::path())?;
            let reader = io::BufReader::new(file);
            let conf = serde_json::from_reader(reader)?;

            Ok(conf)
        }

        pub fn set_contents(contents: &Config) -> Result<(), io::Error> {
            let mut options = fs::OpenOptions::new();
            let mut file = options.write(true).open(self::path())?;
            match file.set_len(0) {
                Ok(_) => (),
                Err(e) => return Err(e),
            };

            let conf = serde_json::to_string(&contents)?;
            write!(file, "{}", conf)
        }
    }
}

use crate::config::config_file;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::process;

fn main() -> Result<(), std::io::Error> {
    let env_key = format!("{}_CONFIG_FILE", env!("CARGO_PKG_NAME"));
    if env::var_os(&env_key).is_none() {
        env::set_var(&env_key, format!(".{}.json", env!("CARGO_PKG_NAME")));
    }

    match config_file::contents() {
        Ok(_) => (),
        Err(e) => {
            match e.kind() {
                std::io::ErrorKind::NotFound => {
                    let mut file = fs::File::create(config_file::path())?;
                    let conf = config::Config::new();
                    write!(file, "{}", serde_json::to_string(&conf).unwrap())?;
                },
                _ => return Err(e),
            }
        }
    };

    let matches = cli::make().get_matches();

    match matches.subcommand() {
        ("add", _) => add(matches),
        ("go", _) => go(matches),
        ("list", _) => list(matches),
        ("open", _) => open(matches),
        ("remove", _) => remove(matches),
        _ => list(matches),
    }
}

fn add(matches: clap::ArgMatches) -> Result<(), io::Error> {
    let mut c = config_file::contents()?;
    let m = matches.subcommand_matches("add").unwrap();

    let n = m.value_of("name").unwrap();
    let d = match fs::canonicalize(m.value_of("dir").unwrap()) {
        Ok(dir) => dir,
        Err(e) => return Err(e),
    };

    let i = c.projects.iter().position(|proj| -> bool {
        proj.name == n
    });

    if i.is_some() {
        return Err(io::Error::from(io::ErrorKind::AlreadyExists));
    }

    c.projects.push(config::Project {
        name: n.to_string(),
        dir: d,
    });

    c.projects.sort_by_key(|proj| proj.name.clone());
    config_file::set_contents(&c)
}

fn go(matches: clap::ArgMatches) -> Result<(), io::Error> {
    let c = config_file::contents()?;
    let m = matches.subcommand_matches("go").unwrap();

    let n = m.value_of("name").unwrap();
    let d = c.projects.iter().find(|&proj| -> bool {
        proj.name == n
    });

    match d {
        Some(project) => {
            let shell = env!("SHELL");
            env::set_current_dir(std::path::Path::new(&project.dir))?;
            let mut command = process::Command::new(shell);
            let mut handle = command.spawn()?;
            handle.wait()?;

            Ok(())
        },
        None => Err(io::Error::from(io::ErrorKind::NotFound)),
    }
}

fn list(matches: clap::ArgMatches) -> Result<(), io::Error> {
    let c = config_file::contents()?;

    let p = match matches.subcommand_matches("list") {
        Some(matches) => matches.values_of("name"),
        None => None,
    };

    let mut ret = Ok(());

    match p {
        Some(projects) => {
            for project in projects {
                let p = c.projects.iter().find(|&proj| -> bool {
                    proj.name == project
                });

                match p {
                    Some(project) => {
                        println!("{}: {}", project.name, project.dir.to_string_lossy());
                    },
                    None => ret = Err(io::Error::from(io::ErrorKind::NotFound)),
                }
            }
        },
        None => {
            let projects = c.projects.iter();

            for project in projects {
                println!("{}: {}", project.name, project.dir.to_string_lossy());
            }
        }
    };

    ret
}

fn open(matches: clap::ArgMatches) -> Result<(), io::Error> {
    let c = config_file::contents()?;
    let m = matches.subcommand_matches("open").unwrap();

    let n = m.value_of("name").unwrap();
    let d = c.projects.iter().find(|&proj| -> bool {
        proj.name == n
    });

    match d {
        Some(project) => {
            let mut command = process::Command::new("code");
            command.arg(&project.dir);

            let mut handle = command.spawn()?;
            handle.wait()?;

            Ok(())
        },
        None => Err(io::Error::from(io::ErrorKind::NotFound)),
    }
}

fn remove(matches: clap::ArgMatches) -> Result<(), io::Error> {
    let mut c = config_file::contents()?;
    let m = matches.subcommand_matches("remove").unwrap();

    let ns = m.values_of("name").unwrap();

    for n in ns {
        let i = c.projects.iter().position(|proj| -> bool {
            proj.name == n
        });

        match i {
            Some(index) => {
                c.projects.remove(index);
                match config_file::set_contents(&c) {
                    Ok(_) => (),
                    Err(e) => return Err(e),
                };
            },
            None => return Err(io::Error::from(io::ErrorKind::NotFound)),
        };
    }

    Ok(())
}
