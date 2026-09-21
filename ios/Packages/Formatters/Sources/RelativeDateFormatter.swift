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
}

private extension RelativeDateFormatter {
    func formatter(dateStyle: DateFormatter.Style, timeStyle: DateFormatter.Style, relative: Bool = false) -> DateFormatter {
        let formatter = DateFormatter()
        formatter.locale = calendar.locale
        formatter.timeZone = calendar.timeZone
        formatter.dateStyle = dateStyle
        formatter.timeStyle = timeStyle
        formatter.doesRelativeDateFormatting = relative
        return formatter
    }
}
