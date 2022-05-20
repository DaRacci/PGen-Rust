mod asset;
mod generator;
mod rules;
mod transformation;

use crate::generator::Generator;
use crate::rules::Rules;
use crate::transformation::Transformation;
use clap::{arg, command, Arg, ArgMatches, Command};
use log::set_max_level;
use simplelog::{debug, error, info, ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger};
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
    if matches.is_present("DEBUG") {
        set_max_level(LevelFilter::Debug);
    }
    if let Some(supplied_rules) = pass_supplied(&matches) {
        rules = supplied_rules;
    }
    pass_args(&mut rules, &matches);
    rules.sanity_checks();

    debug!("Final rule set: {:?}", rules);

    let mut generator = Generator::new(rules);
    let passwords = generator.generate();

    info!("Ask and thou shall receive, here be thine passwords!\n{}", passwords.join("\n"));
}

fn pass_supplied(matches: &ArgMatches) -> Option<Rules> {
    let subcommand = matches.subcommand().unwrap().1;
    let path: PathBuf = if let Some(str) = subcommand.value_of("CONFIG") {
        let mut path = PathBuf::from(str);
        debug!("Trying to use path: {}", path.display());
        if !path.exists() {
            path = Path::new(env::current_dir().unwrap().to_str().unwrap()).join(path);
            debug!("Trying to use path: {}", path.display());
            if !path.exists() {
                error!("File {} does not exist", str);
                process::exit(1);
            }
        }
        path
    } else {
        None?
    };

    debug!("Trying to read file: {}", path.display());

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
            debug!("From supplied rules: {:?}", rules);
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
    matches.value_of("WORDS").map(|words| args.insert("words", words));
    matches.value_of("MIN_LENGTH").map(|min_length| args.insert("min_length", min_length));
    matches.value_of("MAX_LENGTH").map(|max_length| args.insert("max_length", max_length));
    matches.value_of("DIGITS_BEFORE").map(|digits_before| args.insert("digits_before", digits_before));
    matches.value_of("DIGITS_AFTER").map(|digits_after| args.insert("digits_after", digits_after));
    matches.value_of("AMOUNT").map(|amount| args.insert("amount", amount));
    matches.value_of("SEPARATOR_CHAR").map(|separator_char| args.insert("separator_char", separator_char));
    matches.value_of("SEPARATOR_ALPHABET").map(|separator_alphabet| args.insert("separator_alphabet", separator_alphabet));
    matches.value_of("TRANSFORM").map(|transform| args.insert("transform", transform));
    if matches.is_present("MATCH_RANDOM_CHAR") {
        rules.match_random_char = false
    }

    debug!("Supplied arguments {:?}", args);

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
    CombinedLogger::init(vec![
        TermLogger::new(LevelFilter::Debug, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
        WriteLogger::new(LevelFilter::max(), Config::default(), File::create("pgen.log").unwrap()),
    ])
    .unwrap();

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
        debug!("Created config file {}", config_file.display());

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

    debug!("Loaded config from def path: {:?}", toml);

    return toml;
}
