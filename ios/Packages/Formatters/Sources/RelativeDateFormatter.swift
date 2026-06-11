// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public enum RelativeDateFormatterType: Sendable {
    case dateTime
    case date
}

public struct RelativeDateFormatter: Sendable {
    private let type: RelativeDateFormatterType
    public let calendar: Calendar

    public init(
        type: RelativeDateFormatterType = .dateTime,
        locale: Locale = .current,
        timeZone: TimeZone = .current,
    ) {
        self.type = type
        var calendar = Calendar.current
        calendar.locale = locale
        calendar.timeZone = timeZone
        self.calendar = calendar
    }

    public func string(from date: Date) -> String {
        switch type {
        case .dateTime:
            guard calendar.isDateInToday(date) || calendar.isDateInYesterday(date) else {
                return formatter(dateStyle: .long, timeStyle: .short).string(from: date)
            }
            let relative = formatter(dateStyle: .medium, timeStyle: .none, relative: true).string(from: date)
            let time = formatter(dateStyle: .none, timeStyle: .short).string(from: date)
            return "\(relative), \(time)"
        case .date:
            return formatter(dateStyle: .medium, timeStyle: .none).string(from: date)
        }
    }

    public func string(fromTimestampValue value: String) -> String {
        guard let date = date(fromTimestampValue: value) else {
            return value
        }
        return string(from: date)
    }
}

private extension RelativeDateFormatter {
    static let iso8601StrategyWithFractionalSeconds = Date.ISO8601FormatStyle(includingFractionalSeconds: true)
        .year()
        .month()
        .day()
        .time(includingFractionalSeconds: true)
        .timeZone(separator: .omitted)

    static let iso8601Strategy = Date.ISO8601FormatStyle()
        .year()
        .month()
        .day()
        .time(includingFractionalSeconds: false)
        .timeZone(separator: .omitted)

    func formatter(dateStyle: DateFormatter.Style, timeStyle: DateFormatter.Style, relative: Bool = false) -> DateFormatter {
        let formatter = DateFormatter()
        formatter.locale = calendar.locale
        formatter.timeZone = calendar.timeZone
        formatter.dateStyle = dateStyle
        formatter.timeStyle = timeStyle
        formatter.doesRelativeDateFormatting = relative
        return formatter
    }

    func date(fromTimestampValue value: String) -> Date? {
        if let timestamp = TimeInterval(value) {
            return Date(timeIntervalSince1970: timestamp)
        }

        if let date = try? Self.iso8601StrategyWithFractionalSeconds.parse(value) {
            return date
        }

        return try? Self.iso8601Strategy.parse(value)
    }
}
