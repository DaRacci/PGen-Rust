mod generator;
mod rules;

use crate::generator::Generator;
use crate::rules::Rules;
use clap::{arg, command, Command};
use log::{debug, error};
use std::fs::File;
use std::path::Path;
use std::{env, process};

fn main() {
    let matches = command!()
        .propagate_version(true)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .args([
            arg!(-w --words [WORDS] "How many words should be in this password."),
            arg!(-m --minLength [MIN_LENGTH] "Minimum length of the words"),
            arg!(-n --maxLength [MAX_LENGTH] "Maximum length of the words"),
            arg!(-t --transform [TRANSFORM] "What transformation mode to use, Options are [NONE, CAPITALIZE, CAPITALISE_ALL_BUT_FIRST_LETTER, UPPERCASE, LOWERCASE, RANDOM]"),
            arg!(-s --separatorChar [SEPARATOR_CHAR] "Leave blank or 'none' for no split, 'random' to use randomised characters or use any other UTF-8 compliant character for between words."),
            arg!(-c --matchRandomChar [MATCH_RANDOM_CHAR] "Instead of everyone separator being random they will all use the same one random char."),
            arg!(-r --separatorAlphabet [SEPARATOR_ALPHABET] "Defines the random alphabet used between words."),
            arg!(-b --digitsBefore [DIGITS_BEFORE] "Sets how may digits should be before the password."),
            arg!(-a --digitsAfter [DIGITS_AFTER] "Sets how many digits should be after the password."),
            arg!(--amount [AMOUNT] "The amount of passwords to generate."),
            arg!(-d --debug [DEBUG] "Enables debug mode."),
        ])
        .subcommand(
            Command::new("generate")
                .about("Generate some new passwords.")
                .arg(arg!([CONFIG] "The config file to use."))
        )
        .get_matches();

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

            matches.value_of("MATCH_RANDOM_CHAR").then(|match_random_char| match match_random_char.parse() {
                Ok(bool) => rules.match_random_char = bool,
                Err(_) => {
                    error!("Couldn't parse {} as a boolean.\nExiting...", match_random_char);
                    process::exit(1);
                }
            });

            matches.value_of("TRANSFORM").then(|transform| rules.transform = Box::from(transform));
            matches
                .value_of("SEPARATOR_CHAR")
                .then(|separator_char| rules.separator_char = Box::from(separator_char));
            matches
                .value_of("SEPARATOR_ALPHABET")
                .then(|separator_alphabet| rules.separator_alphabet = Box::from(separator_alphabet));

            let mut generator = Generator::new(rules);
            let passwords = generator.generate();

            println!("{}", passwords.join("\n"));
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
