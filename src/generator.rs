use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rust_embed::EmbeddedFile;
use serde_json::{Map, Value};
use simplelog::{debug, error};
use std::process;

use crate::asset::Asset;
use crate::rules::Rules;

pub struct Generator {
    pub rules: Rules,
    selected_char: Option<char>,
    seed: StdRng,
    map: Map<String, Value>,
}

impl Generator {
    pub fn new(rules: Rules) -> Generator {
        debug!("Creating new generator");

        let asset: EmbeddedFile = Asset::get("words.json").unwrap();
        let str = std::str::from_utf8(asset.data.as_ref()).unwrap();
        let parsed: Value = serde_json::from_str(&str).unwrap();
        let map = parsed.as_object().unwrap().clone();

        Generator {
            rules,
            selected_char: None,
            seed: StdRng::from_entropy(),
            map,
        }
    }

    pub fn generate(&mut self) -> Vec<String> {
        let passwords = Vec::with_capacity(self.rules.amount);
        // TODO: Holy shit this is ugly
        let max_length = ((self.rules.max_length * self.rules.amount)
            + (self.rules.digits_before + self.rules.digits_after)
            + (match self.rules.separator_char.len() {
                0 => 0,
                _ => {
                    self.rules.words - 1
                        + match self.rules.digits_before {
                            0 => 0,
                            _ => 1,
                        }
                }
            })) as usize;

        for _ in 0..self.rules.amount {
            let mut password = String::with_capacity(max_length);
            let words = self.get_words();
            let transformed_words = self.transform_words(&words);

            match self.rules.digits_before {
                0 => {}
                digits => {
                    password.push_str(&*self.get_digits(digits));
                    self.get_separator().map(|c| password.push(c));
                }
            }

            password.push_str(&*self.add_separators(&transformed_words));

            match self.rules.digits_after {
                0 => {}
                digits => {
                    self.get_separator().map(|c| password.push(c));
                    password.push_str(&*self.get_digits(digits));
                }
            }

            self.selected_char = None; // Reset for each password.
        }

        return passwords;
    }

    fn get_words(&mut self) -> Vec<String> {
        let mut words: Vec<String> = Vec::with_capacity(self.rules.amount);
        for _ in 0..self.rules.amount {
            let length = self.seed.gen_range(self.rules.min_length..self.rules.max_length);
            let clamped = length.clamp(3, 9);
            let array = self.map.get(clamped.to_string().as_str()).unwrap();
            let word = array.get(self.seed.gen_range(0..array.as_array().unwrap().len())).unwrap();
            words.push(word.as_str().unwrap().to_string());
        }
        words
    }

    fn get_digits(&mut self, int: usize) -> String {
        let mut digits = String::new();
        for _ in 0..int {
            let digit: u8 = self.seed.gen_range(0..9);
            digits.push(digit as char);
        }
        digits
    }

    fn transform_words(&mut self, words: &Vec<String>) -> Vec<String> {
        let mut transformed_words: Vec<String> = Vec::with_capacity(words.len());

        match &*self.rules.transform {
            "NONE" => {}
            "CAPITALISE" => words.iter().for_each(|word| {
                let mut c = word.chars();
                let str = match c.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + c.as_str(),
                };
                transformed_words.push(str);
            }),
            "UPPERCASE_ALL_BUT_FIRST" => words.iter().for_each(|word| {
                let uppercase = word.to_uppercase();
                let mut c = uppercase.chars();
                let str = match c.next() {
                    None => String::new(),
                    Some(first) => first.to_lowercase().collect::<String>() + c.as_str(),
                };
                transformed_words.push(str);
            }),
            "UPPERCASE" => words.iter().for_each(|word| {
                transformed_words.push(word.to_uppercase());
            }),
            "RANDOM" => words.iter().for_each(|word| {
                let mut builder = String::new();
                for char in word.chars() {
                    let new = if self.seed.gen::<bool>() {
                        char.to_uppercase().to_string()
                    } else {
                        char.to_lowercase().to_string()
                    };
                    builder.push_str(&*new);
                }
                transformed_words.push(builder);
            }),
            "ALTERNATING" => words.iter().for_each(|word| {
                let mut builder = String::new();
                for (i, char) in word.chars().enumerate() {
                    let new = if i % 2 == 0 {
                        char.to_uppercase().to_string()
                    } else {
                        char.to_lowercase().to_string()
                    };
                    builder.push_str(&*new);
                }
                transformed_words.push(builder);
            }),
            _ => {
                error!("Unexpected transform type: {}", &*self.rules.transform);
                process::exit(3);
            }
        }

        for word in words {
            let mut transformed_word = String::new();

            for char in word.chars() {
                transformed_word.push(char);
            }

            transformed_words.push(transformed_word);
        }

        transformed_words
    }

    fn get_rand_char(&mut self) -> Option<char> {
        let chars = self.rules.separator_alphabet.chars().collect::<Vec<char>>();
        if chars.len() <= 0 {
            return None;
        }
        Some(chars[self.seed.gen::<usize>() % chars.len()].clone())
    }

    fn get_separator(&mut self) -> Option<char> {
        match &*self.rules.separator_char {
            "NONE" => None,
            "RANDOM" => {
                if self.rules.match_random_char {
                    if self.selected_char.is_none() {
                        self.selected_char = self.get_rand_char()
                    }
                    self.selected_char
                } else {
                    self.get_rand_char()
                }
            }
            _ => self.rules.separator_char.chars().min(),
        }
    }

    fn add_separators(&mut self, words: &Vec<String>) -> String {
        let mut builder = String::new();
        let mut itr = words.iter();
        while let Some(word) = itr.next() {
            if itr.len() > 0 {
                let char = self.get_separator();
                if char.is_some() {
                    builder.push(char.unwrap())
                }
            }
            builder.push_str(word);
        }
        builder
    }
}
