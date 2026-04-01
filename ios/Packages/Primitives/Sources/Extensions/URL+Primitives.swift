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

    func queryValue(for name: String) -> String? {
        URLComponents(url: self, resolvingAgainstBaseURL: false)?
            .queryItems?
            .first { $0.name == name }?
            .value
    }

    func queryValue<T: LosslessStringConvertible>(for name: String) -> T? {
        queryValue(for: name).flatMap { T($0) }
    }

    func isDomainAllowed(_ allowedDomains: [String]) -> Bool {
        guard let host = host?.lowercased() else {
            return false
        }
        return allowedDomains.contains { domain in
            host == domain || host.hasSuffix(".\(domain)")
        }
    }

    func toWebSocketURL() -> URL {
        guard var components = URLComponents(url: self, resolvingAgainstBaseURL: false) else {
            return self
        }

        components.scheme = components.scheme?
            .lowercased()
            .replacingOccurrences(of: "http", with: "ws")
        return components.url ?? self
    }
}
