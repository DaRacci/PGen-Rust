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
use clap::{arg, command, Arg, ArgMatches, Command};
use simplelog::{debug, error, ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{env, fs, process};
use strum::IntoEnumIterator;

fn main() {
    let mut rules = init();
    let matches = get_cli();
    start_logger(matches.is_present("DEBUG"));
    pass_supplied(&matches);
    pass_args(&mut rules, &matches);
    rules.sanity_checks();

    debug!("{:?}", rules);

    let mut generator = Generator::new(rules);
    let passwords = generator.generate();

    info!("Ask and thou shall receive, here be thine passwords!\n{}", passwords.join("\n"));
}

pub trait OptionExt<T> {
    fn then<F, R>(self, f: F)
    where
        R: From<T>,
        F: FnOnce(T) -> Option<R>;
}

impl<T> OptionExt<T> for Option<T> {
    fn then<F, R>(self, f: F)
    where
        R: From<T>,
        F: FnOnce(T) -> Option<R>,
    {
        if self.is_some() {
            f(self.unwrap());
        }
    }
}

fn pass_supplied(matches: &ArgMatches) -> Option<Rules> {
    let subcommand = matches.subcommand().unwrap().1;
    let path: PathBuf = if let Some(str) = subcommand.value_of("CONFIG") {
        let mut path = PathBuf::from(str);
        if !path.exists() {
            path = Path::new(env::current_dir().unwrap().to_str().unwrap()).join(path);
            if !path.exists() {
                error!("File {} does not exist", str);
                process::exit(1);
            }
        }
        path
    } else {
        None?
    };

    if !path.exists() {
        error!("{} does not exist.", path.display());
        process::exit(1);
    } else if !path.is_file() {
        error!("{} is not a file.", path.display());
        process::exit(1);
    }

    let string = fs::read_to_string(&path).unwrap_or_else(|err| {
        error!("Couldn't read {}: {}", path.display(), err);
        process::exit(1);
    });

    match toml::from_str::<Rules>(&string) {
        Ok(rules) => {
            rules.sanity_checks();
            Some(rules)
        }
        Err(err) => {
            error!("Couldn't parse {}: {}", path.display(), err);
            process::exit(1);
        }
    }
}

fn pass_args(rules: &mut Rules, matches: &ArgMatches) {
    let mut args = HashMap::new();
    matches.value_of("WORDS").then(|words| args.insert("words", words));
    matches.value_of("MIN_LENGTH").then(|min_length| args.insert("min_length", min_length));
    matches.value_of("MAX_LENGTH").then(|max_length| args.insert("max_length", max_length));
    matches.value_of("DIGITS_BEFORE").then(|digits_before| args.insert("digits_before", digits_before));
    matches.value_of("DIGITS_AFTER").then(|digits_after| args.insert("digits_after", digits_after));
    matches.value_of("AMOUNT").then(|amount| args.insert("amount", amount));
    matches.value_of("SEPARATOR_CHAR").then(|separator_char| args.insert("separator_char", separator_char));
    matches.value_of("SEPARATOR_ALPHABET").then(|separator_alphabet| args.insert("separator_alphabet", separator_alphabet));
    matches.value_of("TRANSFORM").then(|transform| args.insert("transform", transform));
    if matches.is_present("MATCH_RANDOM_CHAR") {
        rules.match_random_char = false
    }

    for (arg, value) in args {
        match arg {
            "words" => rules.words = unwrap_or_exit(&value),
            "min_length" => rules.min_length = unwrap_or_exit(&value),
            "max_length" => rules.max_length = unwrap_or_exit(&value),
            "digits_before" => rules.digits_before = unwrap_or_exit(&value),
            "digits_after" => rules.digits_after = unwrap_or_exit(&value),
            "amount" => rules.amount = unwrap_or_exit(&value),
            "separator_char" => rules.separator_char = Box::from(value),
            "separator_alphabet" => rules.separator_alphabet = Box::from(value),
            "transform" => rules.transform = Box::from(value),
            "match_random_char" => rules.match_random_char = unwrap_or_exit(&value),
            _ => {}
        }
    }
}

fn unwrap_or_exit<T>(str: &str) -> T
where
    T: FromStr,
{
    match str.parse::<T>() {
        Ok(value) => value,
        Err(_) => {
            error!("Couldn't parse {} as {}", str, stringify!(T));
            process::exit(1);
        }
    }
}

fn start_logger(debug: bool) {
    CombinedLogger::init(vec![
        TermLogger::new(if debug { LevelFilter::Debug } else { LevelFilter::Info }, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
        WriteLogger::new(LevelFilter::max(), Config::default(), File::create("pgen.log").unwrap()),
    ])
    .unwrap();
}

fn get_cli() -> ArgMatches {
    return command!()
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
        .subcommand(Command::new("generate").about("Generate some new passwords.").arg(arg!([CONFIG] "The config file to use.")))
        .get_matches();
}

fn init() -> Rules {
    match env::consts::OS {
        "windows" | "linux" | "macos" => {
            let target_dir = dirs::config_dir().unwrap().join("PGen");

            if !target_dir.exists() {
                fs::create_dir(&target_dir).unwrap_or_else(|err| {
                    error!("Couldn't create directory {}: {}", target_dir.display(), err);
                    process::exit(1);
                });
            }

            return get_config(&target_dir);
        }
        _ => {
            error!("Unsupported OS.");
            process::exit(1);
        }
    }
}

fn get_config(target_dir: &Path) -> Rules {
    let config_file = target_dir.join("PGen.conf");
    if !config_file.exists() {
        let mut file = File::create(&config_file).unwrap_or_else(|err| {
            error!("Couldn't create file {}: {}", config_file.display(), err);
            process::exit(1);
        });
        let string = toml::ser::to_string_pretty(&Rules::default()).unwrap();
        file.write_all(string.as_bytes()).unwrap_or_else(|err| {
            error!("Couldn't write to file {}: {}", config_file.display(), err);
            process::exit(1);
        });
        return Rules::default();
    }

    if !config_file.is_file() {
        error!("{} is not a file.", config_file.display());
        process::exit(1);
    }

    let string = fs::read_to_string(&config_file).unwrap_or_else(|err| {
        error!("Couldn't read file {}: {}", config_file.display(), err);
        process::exit(1);
    });

    let toml = toml::from_str::<Rules>(&string).unwrap_or_else(|err| {
        error!("Couldn't parse file {}: {}", config_file.display(), err);
        process::exit(1);
    });

    toml.sanity_checks();
    return toml;
}
