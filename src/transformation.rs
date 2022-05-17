extern crate strum;
extern crate strum_macros;

use strum_macros::EnumIter;

#[derive(Debug, EnumIter)]
pub enum Transformation {
    NONE,
    CAPITALIZE,
    ALL_EXCEPT_FIRST,
    UPPERCASE,
    LOWERCASE,
    RANDOM,
}
