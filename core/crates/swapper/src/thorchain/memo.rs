#[derive(Debug, Clone, PartialEq)]
pub struct ThorchainMemo {
    pub tx_type: String,
    pub asset: String,
    pub address: String,
    pub affiliate: Option<String>,
    pub affiliate_bps: Option<u32>,
}

impl ThorchainMemo {
    pub fn parse(memo: &str) -> Option<Self> {
        if memo.is_empty() {
            return None;
        }

        let parts: Vec<&str> = memo.split(':').collect();
        if parts.len() < 3 {
            return None;
        }

        Some(ThorchainMemo {
            tx_type: parts[0].to_string(),
            asset: parts[1].to_string(),
            address: parts[2].to_string(),
            affiliate: parts.get(4).filter(|affiliate| !affiliate.is_empty()).map(ToString::to_string),
            affiliate_bps: parts.get(5).and_then(|bps| bps.parse().ok()),
        })
    }

    pub fn is_swap(memo: &str) -> bool {
        Self::parse(memo).is_some_and(|m| m.tx_type == "=" || m.tx_type == "s")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_complete_memo() {
        let memo = "=:ETH.USDT:0x858734a6353C9921a78fB3c937c8E20Ba6f36902:1635978e6/1/0:-_/ll:0/150";
        let parsed = ThorchainMemo::parse(memo).unwrap();

        assert_eq!(parsed.tx_type, "=");
        assert_eq!(parsed.asset, "ETH.USDT");
        assert_eq!(parsed.address, "0x858734a6353C9921a78fB3c937c8E20Ba6f36902");
        assert_eq!(parsed.affiliate.as_deref(), Some("-_/ll"));
        assert_eq!(parsed.affiliate_bps, None);
    }

    #[test]
    fn test_parse_simple_swap() {
        let memo = "=:ETH:0x858734a6353C9921a78fB3c937c8E20Ba6f36902";
        let parsed = ThorchainMemo::parse(memo).unwrap();

        assert_eq!(parsed.tx_type, "=");
        assert_eq!(parsed.asset, "ETH");
        assert_eq!(parsed.address, "0x858734a6353C9921a78fB3c937c8E20Ba6f36902");
        assert_eq!(parsed.affiliate, None);
        assert_eq!(parsed.affiliate_bps, None);
    }

    #[test]
    fn test_parse_bitcoin_memo() {
        let memo = "=:BTC:bc1qaddress:0/1/0:affiliate:150";
        let parsed = ThorchainMemo::parse(memo).unwrap();

        assert_eq!(parsed.address, "bc1qaddress");
        assert_eq!(parsed.affiliate.as_deref(), Some("affiliate"));
        assert_eq!(parsed.affiliate_bps, Some(150));
    }

    #[test]
    fn test_is_swap() {
        assert!(ThorchainMemo::is_swap("=:ETH.USDT:0x858734a6353C9921a78fB3c937c8E20Ba6f36902:1635978e6/1/0"));
        assert!(ThorchainMemo::is_swap("s:ETH:0x858734a6353C9921a78fB3c937c8E20Ba6f36902"));
        assert!(ThorchainMemo::is_swap("=:BTC:bc1qaddress:0/1/0:affiliate:150"));
        assert!(!ThorchainMemo::is_swap(""));
        assert!(!ThorchainMemo::is_swap("WITHDRAW:ETH.ETH:100"));
        assert!(!ThorchainMemo::is_swap("ADD:ETH.ETH:0x123"));
    }

    #[test]
    fn test_parse_invalid_memos() {
        assert!(ThorchainMemo::parse("").is_none());
        assert!(ThorchainMemo::parse("invalid").is_none());
        assert!(ThorchainMemo::parse("=:").is_none());
        assert!(ThorchainMemo::parse("=:ETH").is_none());
    }
}
