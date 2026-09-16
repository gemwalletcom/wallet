// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemDayBoundaries
import GemstonePrimitives

public struct TransactionDateFormatter {
    private static let sectionFormatter: DateFormatter = {
        let formatter = DateFormatter()
        formatter.timeStyle = .none
        formatter.dateStyle = .long
        return formatter
    }()

    private static let rowFormatter: DateFormatter = {
        let formatter = DateFormatter()
        formatter.timeStyle = .short
        formatter.dateStyle = .long
        return formatter
    }()

    private static let rowTimeFormatter: DateFormatter = {
        let formatter = DateFormatter()
        formatter.timeStyle = .short
        formatter.dateStyle = .none
        return formatter
    }()

    private let date: Date
    private let boundaries: GemDayBoundaries

    public init(date: Date, boundaries: GemDayBoundaries = .current) {
        self.date = date
        self.boundaries = boundaries
    }

    public var section: String {
        switch dayLabel {
        case let .some(label): label
        case .none: Self.sectionFormatter.string(from: date)
        }
    }

    public var row: String {
        switch dayLabel {
        case let .some(label): String(format: "%@, %@", label, Self.rowTimeFormatter.string(from: date))
        case .none: Self.rowFormatter.string(from: date)
        }
    }

    private var dayLabel: String? {
        boundaries.label(day: date.gemDay).title
    }
}
