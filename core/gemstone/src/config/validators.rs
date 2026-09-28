use std::collections::HashMap;

use primitives::Chain;

pub fn get_validators() -> HashMap<String, Vec<String>> {
    [
        (
            Chain::Cosmos.to_string(),
            vec![
                "cosmosvaloper1tflk30mq5vgqjdly92kkhhq3raev2hnz6eete3".to_string(),
                "cosmosvaloper1fhr7e04ct0zslmkzqt9smakg3sxrdve6ulclj2".to_string(),
                "cosmosvaloper1hjadhj9nqzpye2vkmkz4thahhd0z8dh3udhq74".to_string(),
            ],
        ),
        (
            Chain::Osmosis.to_string(),
            vec!["osmovaloper1wgmdcxzp49vjgrqusgcagq6qefk4mtjv5c0k7q".to_string(), "osmovaloper1e893vrtzzp6zyzs80tqg52j2vdckzqrdjrjva5".to_string()],
        ),
        (
            Chain::Celestia.to_string(),
            vec!["celestiavaloper1eualhqh07w7p45g45hvrjagkcxsfnflzdw5jzg".to_string(), "celestiavaloper1dlsl4u42ycahzjfwc6td6upgsup9tt7cz8vqm4".to_string()],
        ),
        (Chain::Injective.to_string(), vec!["injvaloper134dct56cq5v7uerxcy2cn4m06mqf4dxrlgpp24".to_string()]),
        (
            Chain::Sei.to_string(),
            vec!["seivaloper1ummny4p645xraxc4m7nphf7vxawfzt3p5hn47t".to_string(), "seivaloper1eqgnd7ey0hnha8rrfukjrsawulhna0zagcg6a4".to_string()],
        ),
        (
            Chain::Sui.to_string(),
            vec![
                "0xbba318294a51ddeafa50c335c8e77202170e1f272599a2edc40592100863f638".to_string(),
                "0x9b8b11c9b2336d35f2db8d5318ff32de51b85857f0e53a5c31242cf3797f4be4".to_string(),
            ],
        ),
        (
            Chain::Solana.to_string(),
            vec![
                "9QU2QSxhb24FUX3Tu2FpczXjpK3VYrvRudywSZaM29mF".to_string(),
                "4PsiLMyoUQ7QRn1FFiFCvej4hsUTFzfvJnyN4bj1tmSN".to_string(),
                "2Y2opv8Kq8zFATg6ipqb2AjgCf18tkv1CLMLXQGif2NH".to_string(),
            ],
        ),
        (Chain::Monad.to_string(), vec!["9".to_string(), "10".to_string()]),
    ]
    .iter()
    .cloned()
    .collect::<HashMap<_, _>>()
}
