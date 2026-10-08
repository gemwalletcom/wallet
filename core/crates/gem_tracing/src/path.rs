const MAX_SEGMENT_LENGTH: usize = 24;

pub fn redact(path: &str) -> String {
    path.split_once('?')
        .map_or(path, |(path, _)| path)
        .split('/')
        .map(|segment| {
            if !segment.is_empty() && segment.chars().all(|character| character.is_ascii_digit()) {
                ":number"
            } else if segment.len() > MAX_SEGMENT_LENGTH {
                ":value"
            } else {
                segment
            }
        })
        .collect::<Vec<_>>()
        .join("/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_dynamic_segments() {
        let cases = [
            ("/wallet/triggerconstantcontract", "/wallet/triggerconstantcontract"),
            ("/api/abcdefghijklmnopqrstuvwx", "/api/abcdefghijklmnopqrstuvwx"),
            ("/api/abcdefghijklmnopqrstuvwxy", "/api/:value"),
            ("/api/v1/verylongsegmentthatisgreaterthan20characters/data", "/api/v1/:value/data"),
            ("/block/12345/transactions", "/block/:number/transactions"),
            ("/block/12345/tx/67890", "/block/:number/tx/:number"),
            ("/api/v1/data", "/api/v1/data"),
            ("/api//data", "/api//data"),
            ("/api/v2/block/5897744?page=1", "/api/v2/block/:number"),
            ("/thorchain/quote/swap?from=X&to=Y", "/thorchain/quote/swap"),
        ];

        for (input, expected) in cases {
            assert_eq!(redact(input), expected, "failed for {input}");
        }
    }
}
