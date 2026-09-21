use primitives::{Currency, FiatRate, Version};

const CURRENCIES_V1: &[Currency] = &[
    Currency::MXN,
    Currency::CHF,
    Currency::CNY,
    Currency::THB,
    Currency::HUF,
    Currency::AUD,
    Currency::IDR,
    Currency::RUB,
    Currency::ZAR,
    Currency::EUR,
    Currency::NZD,
    Currency::SAR,
    Currency::SGD,
    Currency::BMD,
    Currency::KWD,
    Currency::HKD,
    Currency::JPY,
    Currency::GBP,
    Currency::DKK,
    Currency::KRW,
    Currency::PHP,
    Currency::CLP,
    Currency::TWD,
    Currency::PKR,
    Currency::BRL,
    Currency::CAD,
    Currency::BHD,
    Currency::MMK,
    Currency::VEF,
    Currency::VND,
    Currency::CZK,
    Currency::TRY,
    Currency::INR,
    Currency::ARS,
    Currency::BDT,
    Currency::NOK,
    Currency::USD,
    Currency::LKR,
    Currency::ILS,
    Currency::PLN,
    Currency::NGN,
    Currency::UAH,
    Currency::XDR,
    Currency::MYR,
    Currency::AED,
    Currency::SEK,
];

pub(crate) fn filter_fiat_rates(rates: Vec<FiatRate>, version: &Version) -> Vec<FiatRate> {
    if *version >= Version::new(2, 114, 32) { rates } else { filter_fiat_rates_v1(rates) }
}

pub(crate) fn filter_fiat_rates_v1(rates: Vec<FiatRate>) -> Vec<FiatRate> {
    rates.into_iter().filter(|rate| CURRENCIES_V1.contains(&rate.symbol)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fiat_rates_version() {
        let rates = vec![FiatRate { symbol: Currency::USD, rate: 1.0 }, FiatRate { symbol: Currency::BYN, rate: 3.03 }];
        assert_eq!(filter_fiat_rates_v1(rates.clone()), rates[..1]);
        for version in ["1.0.0", "2.114.31"] {
            assert_eq!(filter_fiat_rates(rates.clone(), &version.parse().unwrap()), rates[..1]);
        }
        for version in ["2.114.32", "2.114.100", "2.115.0", "3.0.0"] {
            assert_eq!(filter_fiat_rates(rates.clone(), &version.parse().unwrap()), rates);
        }
    }
}
