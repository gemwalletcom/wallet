// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

extension ChartPeriod: Identifiable {
    public var id: String { rawValue }
}

public extension ChartPeriod {
    init(id: String) throws {
        if let period = ChartPeriod(rawValue: id) {
            self = period
        } else {
            throw AnyError("invalid chart period: \(id)")
        }
    }

    var duration: Int {
        switch self {
        case .hour:
            60 * 60 // 1 hour
        case .day:
            24 * 60 * 60 // 24 hours
        case .week:
            7 * 24 * 60 * 60 // 7 days
        case .month:
            30 * 24 * 60 * 60 // 30 days
        case .year:
            365 * 24 * 60 * 60 // 365 days
        case .all:
            365 * 5 * 24 * 60 * 60 // 5 years
        }
    }
}
