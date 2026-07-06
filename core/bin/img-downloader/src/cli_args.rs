use clap::{Parser, ValueEnum};

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum ImageSource {
    Coingecko,
    Coinmarketcap,
    Jupiter,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum ImageMode {
    Top,
    Trending,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Image source provider
    #[arg(long, value_enum, default_value = "coingecko")]
    pub source: ImageSource,

    /// Image mode for providers that support multiple feeds
    #[arg(long, value_enum, default_value = "top")]
    pub mode: ImageMode,

    /// Path to save images
    #[arg(short, long)]
    pub folder: Option<String>,

    /// Provider ID. CoinGecko uses coin ID, CoinMarketCap uses coin ID or symbol, Jupiter uses token mint
    #[arg(long, default_value = "")]
    pub id: String,
}
