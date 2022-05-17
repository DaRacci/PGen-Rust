use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Rules {
    pub words: usize,
    pub min_length: usize,
    pub max_length: usize,
    pub transform: Box<str>,
    pub separator_char: Box<str>,
    pub separator_alphabet: Box<str>,
    pub match_random_char: bool,
    pub digits_before: usize,
    pub digits_after: usize,
    pub amount: usize,
}

impl Default for Rules {
    fn default() -> Self {
        Rules {
            words: 2,
            min_length: 5,
            max_length: 7,
            transform: Box::from("CAPITALISE"),
            separator_char: Box::from("RANDOM"),
            separator_alphabet: Box::from("!@$%.&*-+=?:;"),
            match_random_char: true,
            digits_before: 0,
            digits_after: 3,
            amount: 3,
        }
    }
}
