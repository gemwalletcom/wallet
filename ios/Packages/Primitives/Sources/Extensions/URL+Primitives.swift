// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public extension URL {
    func cleanHost() -> String? {
        guard let host else { return host }
        let values = ["www."]
        for value in values {
            if host.hasPrefix(value) {
                return host.replacingOccurrences(of: value, with: "")
            }
        }
        return host
    }

    func appending(queryItems newItems: [URLQueryItem]) -> URL {
        var components = URLComponents(url: self, resolvingAgainstBaseURL: false)!
        var queryItems = components.queryItems ?? []
        queryItems.append(contentsOf: newItems)
        components.queryItems = queryItems
        return components.url!
    }

    func withUTM(source: String) -> URL {
        appending(queryItems: [URLQueryItem(name: "utm_source", value: source)])
    }
}
