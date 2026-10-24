use crate::{Chain, ChainType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressFormatStyle {
    Short,
    Full,
    Extra { extra: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AddressFormatter;

impl AddressFormatter {
    const CONNECTOR: &str = "...";
    const EDGE_CHARS: usize = 5;
    const ZCASH_UNIFIED_PREFIX: &str = "u1";
    const ZCASH_UNIFIED_LEADING_CHARS: usize = 20;

    pub fn format(address: &str, chain: Option<Chain>, style: AddressFormatStyle) -> String {
        match style {
            AddressFormatStyle::Short => Self::truncate(address, chain, 0),
            AddressFormatStyle::Full => address.to_string(),
            AddressFormatStyle::Extra { extra } => Self::truncate(address, chain, extra as usize),
        }
    }

    fn truncate(address: &str, chain: Option<Chain>, extra: usize) -> String {
        let leading = Self::leading_chars(address, chain) + extra;
        let trailing = Self::EDGE_CHARS + extra;
        let chars = address.chars().collect::<Vec<_>>();
        let char_count = chars.len();

        if char_count <= leading + trailing {
            return address.to_string();
        }

        let start = chars.iter().take(leading).copied().collect::<String>();
        let end = chars.iter().skip(char_count - trailing).copied().collect::<String>();
        format!("{start}{}{end}", Self::CONNECTOR)
    }

    fn leading_chars(address: &str, chain: Option<Chain>) -> usize {
        if chain == Some(Chain::Zcash) && address.get(..Self::ZCASH_UNIFIED_PREFIX.len()).is_some_and(|prefix| prefix.eq_ignore_ascii_case(Self::ZCASH_UNIFIED_PREFIX)) {
            return Self::ZCASH_UNIFIED_LEADING_CHARS;
        }
        match chain.map(|chain| chain.chain_type()) {
            Some(ChainType::Ethereum) => Self::EDGE_CHARS + "0x".len(),
            Some(ChainType::Bitcoin | ChainType::Aptos) => Self::EDGE_CHARS + 1,
            Some(_) | None => Self::EDGE_CHARS,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_formatter() {
        assert_eq!(AddressFormatter::format("0x12312321321312", Some(Chain::Ethereum), AddressFormatStyle::Short), "0x12312...21312");
        assert_eq!(AddressFormatter::format("0x12312321321312", Some(Chain::Aptos), AddressFormatStyle::Short), "0x1231...21312");
        assert_eq!(AddressFormatter::format("GLNvG5Ly4cK512oQeJqnwLftwfoPZ4skyDwZWzxorYQ9", Some(Chain::Solana), AddressFormatStyle::Short), "GLNvG...orYQ9");
        assert_eq!(
            AddressFormatter::format("bc1qx2x5cqhymfcnjtg902ky6u5t5htmt7fvqztdsm028hkrvxcl4t2sjtpd9l", Some(Chain::Bitcoin), AddressFormatStyle::Short),
            "bc1qx2...tpd9l"
        );
        assert_eq!(AddressFormatter::format("0x1231232221321312", Some(Chain::Ethereum), AddressFormatStyle::Extra { extra: 2 }), "0x1231232...1321312");
        assert_eq!(AddressFormatter::format("0x12313332321321312", Some(Chain::Aptos), AddressFormatStyle::Extra { extra: 2 }), "0x123133...1321312");
        assert_eq!(AddressFormatter::format("0x1231232221321312", Some(Chain::Ethereum), AddressFormatStyle::Full), "0x1231232221321312");
        assert_eq!(AddressFormatter::format("bc1short", Some(Chain::Bitcoin), AddressFormatStyle::Short), "bc1short");
        assert_eq!(AddressFormatter::format("abcXXmiddleXXcba", Some(Chain::Solana), AddressFormatStyle::Short), "abcXX...XXcba");
        assert_eq!(
            AddressFormatter::format(
                "u1qhwq6evpqq04ga6dn06mexlk6pcarhru4mhpkv63ar7nsmqvk5057lgtuj2yx6nhxlu0wuhzuw74v4q0eskt7gxy2xplq4qe5m560gukxtnx86mu5qljhgygyxzy7ulavg02qc42hxs",
                Some(Chain::Zcash),
                AddressFormatStyle::Short
            ),
            "u1qhwq6evpqq04ga6dn0...42hxs"
        );
        assert_eq!(
            AddressFormatter::format(
                "U1QHWQ6EVPQQ04GA6DN06MEXLK6PCARHRU4MHPKV63AR7NSMQVK5057LGTUJ2YX6NHXLU0WUHZUW74V4Q0ESKT7GXY2XPLQ4QE5M560GUKXTNX86MU5QLJHGYGYXZY7ULAVG02QC42HXS",
                Some(Chain::Zcash),
                AddressFormatStyle::Short
            ),
            "U1QHWQ6EVPQQ04GA6DN0...42HXS"
        );
        assert_eq!(AddressFormatter::format("t1SdZhkr3TmiHttYNL6y4qsK5N7nMDoPHm8", Some(Chain::Zcash), AddressFormatStyle::Short), "t1SdZh...oPHm8");
    }
}
