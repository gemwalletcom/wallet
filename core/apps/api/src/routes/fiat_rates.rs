use primitives::{Currency, FiatRate, Version};

pub fn filter_fiat_rates(rates: Vec<FiatRate>, version: &Version) -> Vec<FiatRate> {
    rates.into_iter().filter(|rate| minimum_version(&rate.symbol).is_none_or(|minimum| *version >= minimum)).collect()
}

pub fn filter_fiat_rates_v1(rates: Vec<FiatRate>) -> Vec<FiatRate> {
    rates.into_iter().filter(|rate| minimum_version(&rate.symbol).is_none()).collect()
}

fn minimum_version(currency: &Currency) -> Option<Version> {
    match currency {
        Currency::RON | Currency::GEL | Currency::AMD | Currency::MDL => Some(Version::new(2, 114, 84)),
        Currency::BYN | Currency::KZT | Currency::UZS | Currency::EGP | Currency::KES | Currency::COP | Currency::MAD | Currency::GHS | Currency::PEN => Some(Version::new(2, 114, 32)),
        Currency::MXN
        | Currency::CHF
        | Currency::CNY
        | Currency::THB
        | Currency::HUF
        | Currency::AUD
        | Currency::IDR
        | Currency::RUB
        | Currency::ZAR
        | Currency::EUR
        | Currency::NZD
        | Currency::SAR
        | Currency::SGD
        | Currency::BMD
        | Currency::KWD
        | Currency::HKD
        | Currency::JPY
        | Currency::GBP
        | Currency::DKK
        | Currency::KRW
        | Currency::PHP
        | Currency::CLP
        | Currency::TWD
        | Currency::PKR
        | Currency::BRL
        | Currency::CAD
        | Currency::BHD
        | Currency::MMK
        | Currency::VEF
        | Currency::VND
        | Currency::CZK
        | Currency::TRY
        | Currency::INR
        | Currency::ARS
        | Currency::BDT
        | Currency::NOK
        | Currency::USD
        | Currency::LKR
        | Currency::ILS
        | Currency::PLN
        | Currency::NGN
        | Currency::UAH
        | Currency::XDR
        | Currency::MYR
        | Currency::AED
        | Currency::SEK => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fiat_rates_version() {
        let rates = vec![
            FiatRate { symbol: Currency::USD, rate: 1.0 },
            FiatRate { symbol: Currency::BYN, rate: 3.03 },
            FiatRate { symbol: Currency::RON, rate: 5.0 },
            FiatRate { symbol: Currency::GEL, rate: 2.5 },
            FiatRate { symbol: Currency::AMD, rate: 400.0 },
            FiatRate { symbol: Currency::MDL, rate: 20.0 },
        ];
        assert_eq!(filter_fiat_rates_v1(rates.clone()), rates[..1]);
        for version in ["0", "0.0", "0.0.0", "1.0.0", "2.114.31"] {
            assert_eq!(filter_fiat_rates(rates.clone(), &version.parse().unwrap()), rates[..1]);
        }
        for version in ["2.114.32", "2.114.82", "2.114.83"] {
            assert_eq!(filter_fiat_rates(rates.clone(), &version.parse().unwrap()), rates[..2]);
        }
        for version in ["2.114.84", "2.114.100", "2.115.0", "3.0.0"] {
            assert_eq!(filter_fiat_rates(rates.clone(), &version.parse().unwrap()), rates);
        }
    }
}
