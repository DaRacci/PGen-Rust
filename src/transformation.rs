extern crate strum;
extern crate strum_macros;

use strum_macros::{EnumIter, EnumString};

#[derive(Debug, EnumIter, EnumString)]
pub enum Transformation {
    NONE,
    CAPITALISE,
    ALL_EXCEPT_FIRST,
    UPPERCASE,
    RANDOM,
    ALTERNATING,
}
