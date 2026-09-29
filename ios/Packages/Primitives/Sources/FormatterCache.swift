// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public enum FormatterCache {
    private nonisolated(unsafe) static let formatters = NSCache<NSString, Formatter>()

    public static func formatter<T: Formatter>(key: String, make: () -> T) -> T {
        let cacheKey = "\(T.self)|\(key)" as NSString
        if let formatter = formatters.object(forKey: cacheKey) as? T {
            return formatter
        }
        let formatter = make()
        formatters.setObject(formatter, forKey: cacheKey)
        return formatter
    }
}
