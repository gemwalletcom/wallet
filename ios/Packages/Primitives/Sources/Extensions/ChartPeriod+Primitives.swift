// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

extension ChartPeriod: Identifiable {
    public var id: String {
        rawValue
    }
}

public extension ChartPeriod {
    init(id: String) throws {
        if let period = ChartPeriod(rawValue: id) {
            self = period
        } else {
            throw AnyError("invalid chart period: \(id)")
        }
    }
}
