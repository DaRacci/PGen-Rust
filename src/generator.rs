use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rust_embed::EmbeddedFile;
use serde_json::{Map, Value};
use simplelog::debug;

use crate::asset::Asset;
use crate::rules::Rules;
use crate::Transformation;

pub struct Generator {
    pub rules: Rules,
    selected_char: Option<char>,
    seed: StdRng,
    map: Map<String, Value>,
}

// TODO: RNG generates the same values quite often.
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
        let mut passwords = Vec::with_capacity(self.rules.amount);
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

        debug!("Generating {} passwords with max length {}", self.rules.amount, max_length);

        for _ in 0..self.rules.amount {
            let mut password = String::with_capacity(max_length);
            let words = self.get_words();
            let transformed_words = self.transform_words(&words);

            match self.rules.digits_before {
                0 => debug!("No digits before"),
                digits => {
                    debug!("Adding {} digits before", digits);
                    password.push_str(&*self.get_digits(digits));
                    self.get_separator().map(|c| password.push(c));
                }
            }

            password.push_str(&*self.add_separators(&transformed_words));

            match self.rules.digits_after {
                0 => debug!("No digits after"),
                digits => {
                    debug!("Adding {} digits after", digits);
                    self.get_separator().map(|c| password.push(c));
                    password.push_str(&*self.get_digits(digits));
                }
            }

            passwords.push(password);
            self.selected_char = None; // Reset for each password.
        }

        return passwords;
    }

    fn get_words(&mut self) -> Vec<String> {
        let mut words: Vec<String> = Vec::with_capacity(self.rules.amount);
        for _ in 0..self.rules.words {
            let length = self.seed.gen_range(self.rules.min_length..self.rules.max_length);
            let array = self.map.get(length.to_string().as_str()).unwrap();
            let word = array.get(self.seed.gen_range(0..array.as_array().unwrap().len())).unwrap();
            words.push(word.as_str().unwrap().to_string());
        }
        debug!("Got {} words", words.len());
        debug!("{:?}", words);

        words
    }

    fn get_digits(&mut self, int: usize) -> String {
        let mut digits = String::new();
        for _ in 0..int {
            let digit: u32 = self.seed.gen_range(0..9);
            digits.push(char::from_digit(digit, 10).unwrap());
        }
        digits
    }

    fn transform_words(&mut self, words: &Vec<String>) -> Vec<String> {
        let mut transformed_words: Vec<String> = Vec::with_capacity(words.len());

        match Transformation::try_from(&*self.rules.transform.to_uppercase()).unwrap() {
            Transformation::NONE => {
                debug!("No transformation, doing nothing.");
                transformed_words = words.clone();
            }
            Transformation::CAPITALISE => words.iter().for_each(|word| {
                let mut c = word.chars();
                let str = match c.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + c.as_str(),
                };
                debug!("Capitalised word {}", str);
                transformed_words.push(str);
            }),
            Transformation::ALL_EXCEPT_FIRST => words.iter().for_each(|word| {
                let uppercase = word.to_uppercase();
                let mut c = uppercase.chars();
                let str = match c.next() {
                    None => String::new(),
                    Some(first) => first.to_lowercase().collect::<String>() + c.as_str(),
                };
                debug!("Uppercase all but first character {}", str);
                transformed_words.push(str);
            }),
            Transformation::UPPERCASE => words.iter().for_each(|word| {
                let uppercase = word.to_uppercase();
                debug!("Uppercase word {}", uppercase);
                transformed_words.push(uppercase);
            }),
            Transformation::RANDOM => words.iter().for_each(|word| {
                let mut builder = String::new();
                for char in word.chars() {
                    let new = if self.seed.gen::<bool>() {
                        char.to_uppercase().to_string()
                    } else {
                        char.to_lowercase().to_string()
                    };
                    builder.push_str(&*new);
                }
                debug!("Randomised uppercase and lowercase word {}", builder);
                transformed_words.push(builder);
            }),
            Transformation::ALTERNATING => words.iter().for_each(|word| {
                let mut builder = String::new();
                for (i, char) in word.chars().enumerate() {
                    let new = if i % 2 == 0 {
                        char.to_uppercase().to_string()
                    } else {
                        char.to_lowercase().to_string()
                    };
                    builder.push_str(&*new);
                }
                debug!("Alternating uppercase and lowercase word {}", builder);
                transformed_words.push(builder);
            }),
        }

        debug!("Transformed words: {:?}", transformed_words);

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
            "NONE" => {
                debug!("No separator char");
                None
            }
            "RANDOM" => {
                debug!("Random separator char");
                if self.rules.match_random_char {
                    debug!("Using the same random char for all separators");
                    if self.selected_char.is_none() {
                        let char = self.get_rand_char();
                        debug!("No random char selected, generating one: {:?}", char);
                        self.selected_char = char
                    }
                    self.selected_char
                } else {
                    let char = self.get_rand_char();
                    debug!("Random char selected: {:?}", char);
                    char
                }
            }
            _ => {
                debug!("Separator char: {}", &*self.rules.separator_char);
                self.rules.separator_char.chars().min()
            }
        }
    }

    fn add_separators(&mut self, words: &Vec<String>) -> String {
        let mut builder = String::new();
        let mut itr = words.iter();

        while itr.len() >= 1 {
            let word = itr.next().unwrap();
            builder.push_str(word);
            if itr.len() > 0 {
                builder.push(self.get_separator().unwrap());
            }
        }

        debug!("Final string: {}", builder);
        builder
    }
}
