// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import Foundation
import Primitives

public struct ChartDateFormatter: Sendable {
    private let relative: RelativeDateFormatter
    private let locale: Locale
    private let timeZone: TimeZone

    public init(
        relative: RelativeDateFormatter = RelativeDateFormatter(),
        locale: Locale = .current,
        timeZone: TimeZone = .current,
    ) {
        self.relative = relative
        self.locale = locale
        self.timeZone = timeZone
    }

    public func string(for date: Date, period: ChartPeriod) -> String {
        switch period {
        case .hour, .day: relative.string(from: date)
        case .week, .month: date.formatted(dateTime.month(.abbreviated).day().hour().minute())
        case .year, .all: date.formatted(dateTime.year().month(.abbreviated).day())
        }
    }

    private var dateTime: Date.FormatStyle {
        Date.FormatStyle(locale: locale, timeZone: timeZone)
    }
}
