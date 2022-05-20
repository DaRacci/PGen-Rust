use strum_macros::{EnumIter, EnumString};

#[allow(non_camel_case_types)]
#[derive(Debug, EnumIter, EnumString)]
pub enum Transformation {
    NONE,
    CAPITALISE,
    ALL_EXCEPT_FIRST,
    UPPERCASE,
    RANDOM,
    ALTERNATING,
}
