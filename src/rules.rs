use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};

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

impl Rules {
    pub fn sanity_checks(&self) -> Result<(), String> {
        if self.words < 1 || self.words > 10 {
            return Err(format!("Words must be within bounds of 1 and 10, received {}", self.words));
        }

        if self.min_length < 3 || self.min_length > 9 {
            return Err(format!("Min length must be within bounds of 3 and 9, received {}", self.min_length));
        }

        if self.max_length < 3 || self.max_length > 9 {
            return Err(format!("Max length must be within bounds of 3 and 9, received {}", self.max_length));
        }

        if self.min_length > self.max_length {
            return Err(format!("Min length must be less than or equal to max length, received {}", self.max_length));
        }

        Ok(())
    }
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

impl Debug for Rules {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Rules")
            .field("words", &self.words)
            .field("min_length", &self.min_length)
            .field("max_length", &self.max_length)
            .field("transform", &self.transform)
            .field("separator_char", &self.separator_char)
            .field("separator_alphabet", &self.separator_alphabet)
            .field("match_random_char", &self.match_random_char)
            .field("digits_before", &self.digits_before)
            .field("digits_after", &self.digits_after)
            .field("amount", &self.amount)
            .finish()
    }
}
