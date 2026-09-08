use clap::{Parser, ValueEnum};

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum ImageSource {
    Coingecko,
    Coinmarketcap,
    Jupiter,
    Dexscreener,
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

    #[arg(long, value_enum, default_value = "top", help = "Image list selection; ignored when --id is supplied")]
    pub mode: ImageMode,

    /// Path to save images
    #[arg(short, long)]
    pub folder: Option<String>,

    #[arg(
        long,
        default_value = "",
        help = "Provider ID: CoinGecko coin ID, CoinMarketCap ID/symbol, Jupiter mint, DexScreener chain_token-address"
    )]
    pub id: String,
}
