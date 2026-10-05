// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemDayBoundaries
import GemstonePrimitives
import Localization
import os

public struct TransactionDateFormatter: Sendable {
    private static let formatters = OSAllocatedUnfairLock<[FormatterKey: DateFormatter]>(initialState: [:])

    private let date: Date?
    private let boundaries: GemDayBoundaries
    private let locale: Locale
    private let timeZone: TimeZone

    public init(
        date: Date?,
        boundaries: GemDayBoundaries = .current,
        locale: Locale = .current,
        timeZone: TimeZone = .current,
    ) {
        self.date = date
        self.boundaries = boundaries
        self.locale = locale
        self.timeZone = timeZone
    }

    public init(
        unixMilliseconds: Int64,
        boundaries: GemDayBoundaries = .current,
        locale: Locale = .current,
        timeZone: TimeZone = .current,
    ) {
        self.init(
            date: unixMilliseconds == 0 ? nil : Date(timeIntervalSince1970: TimeInterval(unixMilliseconds) / 1000),
            boundaries: boundaries,
            locale: locale,
            timeZone: timeZone,
        )
    }

    public var section: String {
        guard let date else { return "" }
        return dayLabel ?? formatter(dateStyle: .long, timeStyle: .none).string(from: date)
    }

    public var row: String {
        guard let date else { return "" }
        guard let dayLabel else {
            return formatter(dateStyle: .long, timeStyle: .short).string(from: date)
        }
        return Localized.Date.dayTime(dayLabel, formatter(dateStyle: .none, timeStyle: .short).string(from: date))
    }

    public var day: String {
        guard let date else { return "" }
        return formatter(dateStyle: .medium, timeStyle: .none).string(from: date)
    }

    private var dayLabel: String? {
        date.flatMap { boundaries.label(day: $0.gemDay(in: timeZone)).title }
    }

    private func formatter(dateStyle: DateFormatter.Style, timeStyle: DateFormatter.Style) -> DateFormatter {
        let key = FormatterKey(locale: locale, hourCycle: locale.hourCycle, timeZone: timeZone, dateStyle: dateStyle, timeStyle: timeStyle)
        return Self.formatters.withLock { formatters in
            if let formatter = formatters[key] {
                return formatter
            }
            let formatter = DateFormatter()
            formatter.locale = locale
            formatter.timeZone = timeZone
            formatter.dateStyle = dateStyle
            formatter.timeStyle = timeStyle
            formatters[key] = formatter
            return formatter
        }
    }
}

private struct FormatterKey: Hashable {
    let locale: Locale
    let hourCycle: Locale.HourCycle
    let timeZone: TimeZone
    let dateStyle: DateFormatter.Style
    let timeStyle: DateFormatter.Style
}
