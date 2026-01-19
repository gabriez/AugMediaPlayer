
use     clap::{Parser, Subcommand}  ;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// The URI of the media to play 
    #[clap(short, long)]
    uri: String,

    /// The origin of the media to play
    #[command(subcommand)]
    origin: OriginType,
}

impl Args {
    pub fn formatted_uri(&self) -> String {
        match self.origin {
            OriginType::File => format!("file://{}", self.uri),
            OriginType::Http => self.uri.clone(),
        }
    }
}

#[derive(Subcommand)]
pub enum OriginType {
    /// The media is a file
    #[clap(name = "file")]
    File,
    /// The media is a HTTP stream
    #[clap(name = "http")]
    Http,
}