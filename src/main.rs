mod asset;
mod generator;
mod rules;
mod transformation;

#[macro_use]
extern crate log;
extern crate simplelog;
extern crate strum_macros;

use crate::generator::Generator;
use crate::rules::Rules;
use crate::transformation::Transformation;
use clap::{arg, command, Arg, Command};
use simplelog::{debug, error, ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger};
use std::fs::File;
use std::path::Path;
use std::{env, process};
use strum::IntoEnumIterator;

fn main() {
    let matches = command!()
        .propagate_version(true)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .args([
            Arg::new("WORDS")
                .help(format!("The number of words to generate for each password (default: {})", Rules::default().words).as_str())
                .takes_value(true)
                .short('w')
                .long("words"),
            Arg::new("MIN_LENGTH")
                .help(format!("The minimum length of each word (default: {}, min: 3)", Rules::default().min_length).as_str())
                .takes_value(true)
                .short('m')
                .long("min-length"),
            Arg::new("MAX_LENGTH")
                .help(format!("The maximum length of each word (default: {}, max: 9)", Rules::default().max_length).as_str())
                .takes_value(true)
                .short('M')
                .long("max-length"),
            Arg::new("DIGITS_BEFORE")
                .help(format!("The number of digits before the words (default: {})", Rules::default().digits_before).as_str())
                .takes_value(true)
                .short('d')
                .long("digits-before"),
            Arg::new("DIGITS_AFTER")
                .help(format!("The number of digits after the words (default: {})", Rules::default().digits_after).as_str())
                .takes_value(true)
                .short('D')
                .long("digits-after"),
            Arg::new("TRANSFORM")
                .help(
                    format!(
                        "What transformation mode to use, Options are {:?} (default: {})",
                        Transformation::iter().collect::<Vec<_>>(),
                        Rules::default().transform
                    )
                    .as_str(),
                )
                .takes_value(true)
                .short('t')
                .long("transform"),
            Arg::new("SEPARATOR_CHAR")
                .help(format!("The character to use to separate the words (default: \"{}\")", Rules::default().separator_char).as_str())
                .takes_value(true)
                .short('s')
                .long("separator-char"),
            Arg::new("SEPARATOR_ALPHABET")
                .help(format!("The array of characters as separators (default: \"{}\")", Rules::default().separator_alphabet).as_str())
                .takes_value(true)
                .short('S')
                .long("separator-alphabet"),
            Arg::new("MATCH_RANDOM_CHAR")
                .help(
                    format!(
                        "Do not use the same random character for each separator rather than a new random each time (default: {})",
                        Rules::default().match_random_char
                    )
                    .as_str(),
                )
                .short('r')
                .long("match-random-char"),
            Arg::new("AMOUNT")
                .help(format!("The number of passwords to generate (default: {})", Rules::default().amount).as_str())
                .takes_value(true)
                .short('a')
                .long("amount"),
            Arg::new("DEBUG").help("Enable debug logging").long("debug"),
        ])
        .subcommand(
            Command::new("generate")
                .about("Generate some new passwords.")
                .arg(arg!([CONFIG] "The config file to use.")),
        )
        .get_matches();

    CombinedLogger::init(vec![
        TermLogger::new(
            if matches.is_present("DEBUG") { LevelFilter::Debug } else { LevelFilter::Info },
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(LevelFilter::max(), Config::default(), File::create("pgen.log").unwrap()),
    ])
    .unwrap();

    let mut rules: Rules;

    match matches.subcommand() {
        Some(("generate", sub_matches)) => {
            rules = match sub_matches.value_of("CONFIG") {
                None => Rules::default(),
                Some(path_str) => {
                    let path = Path::new(path_str);
                    if !path.exists() {
                        error!("The config file {} does not exist.", path.display());
                        process::exit(1);
                    }

                    let attr = path.metadata().unwrap();
                    debug!(
                        "File Attributes: [isFile={},isSymlink={},readOnly={}]",
                        attr.is_file(),
                        attr.is_symlink(),
                        attr.permissions().readonly()
                    );

                    let file = File::open(path).unwrap_or_else(|e| {
                        error!("Couldn't open the config file {}: {}", path.display(), e);
                        process::exit(2);
                    });

                    serde_json::from_reader(file).unwrap_or_else(|e| {
                        error!("Failed to parse config file: {}", e);
                        process::exit(1);
                    })
                }
            };

            matches.value_of("WORDS").then(|words| match words.parse() {
                Ok(number) => rules.words = number,
                Err(_) => {
                    error!("Couldn't parse {} as a usize.\nExiting...", words);
                    process::exit(1);
                }
            });

            matches.value_of("MIN_LENGTH").then(|min_length| match min_length.parse() {
                Ok(number) => rules.min_length = number,
                Err(_) => {
                    error!("Couldn't parse {} as a usize.\nExiting...", min_length);
                    process::exit(1);
                }
            });

            matches.value_of("MAX_LENGTH").then(|max_length| match max_length.parse() {
                Ok(number) => rules.max_length = number,
                Err(_) => {
                    error!("Couldn't parse {} as a usize.\nExiting...", max_length);
                    process::exit(1);
                }
            });

            matches.value_of("MATCH_RANDOM_CHAR").then(|match_random_char| match match_random_char.parse() {
                Ok(number) => rules.match_random_char = number,
                Err(_) => {
                    error!("Couldn't parse {} as a usize.\nExiting...", match_random_char);
                    process::exit(1);
                }
            });

            matches.value_of("DIGITS_BEFORE").then(|digits_before| match digits_before.parse() {
                Ok(number) => rules.digits_before = number,
                Err(_) => {
                    error!("Couldn't parse {} as a usize.\nExiting...", digits_before);
                    process::exit(1);
                }
            });

            matches.value_of("DIGITS_AFTER").then(|digits_after| match digits_after.parse() {
                Ok(number) => rules.digits_after = number,
                Err(_) => {
                    error!("Couldn't parse {} as a usize.\nExiting...", digits_after);
                    process::exit(1);
                }
            });

            matches.value_of("AMOUNT").then(|amount| match amount.parse() {
                Ok(number) => rules.amount = number,
                Err(_) => {
                    error!("Couldn't parse {} as a usize.\nExiting...", amount);
                    process::exit(1);
                }
            });

            if matches.is_present("MATCH_RANDOM_CHAR") {
                rules.match_random_char = false;
            }

            matches.value_of("TRANSFORM").then(|transform| rules.transform = Box::from(transform));
            matches
                .value_of("SEPARATOR_CHAR")
                .then(|separator_char| rules.separator_char = Box::from(separator_char));
            matches
                .value_of("SEPARATOR_ALPHABET")
                .then(|separator_alphabet| rules.separator_alphabet = Box::from(separator_alphabet));

            rules.sanity_checks();

            debug!("{:?}", rules);

            let mut generator = Generator::new(rules);
            let passwords = generator.generate();

            info!("Ask and thou shall receive, here be thine passwords!\n{}", passwords.join("\n"));
        }
        _ => {}
    }
}

pub trait OptionExt<T> {
    fn then<F>(self, f: F)
    where
        F: FnOnce(T) -> ();
}

impl<T> OptionExt<T> for Option<T> {
    fn then<F>(self, f: F)
    where
        F: FnOnce(T) -> (),
    {
        if self.is_some() {
            f(self.unwrap());
        }
    }
}

fn init() {
    match env::consts::OS {
        "windows" => {
            println!("Windows is not supported yet.");
        }
        "macos" => {
            println!("MacOS is not supported yet.");
        }
        "linux" => {
            println!("Linux is not supported yet.");
        }
        other => {
            error!("{} is not supported, please use windows, linux, or macos.", other);
            process::exit(1);
        }
    }
}
