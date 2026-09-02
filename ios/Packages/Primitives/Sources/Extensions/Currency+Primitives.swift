// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public extension Currency {
    static let `default`: Currency = .usd

    init(id: String) throws {
        if let currency = Currency(rawValue: id) {
            self = currency
        } else {
            throw AnyError("invalid currency: \(id)")
        }
    }
}

extension Currency: Identifiable {
    public var id: String {
        rawValue
    }
}
