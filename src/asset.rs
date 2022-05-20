use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "assets"]
pub(crate) struct Asset;
