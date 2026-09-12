// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.DurationFormatter
import struct Gemstone.GemDurationPart
import enum Gemstone.GemDurationUnit

public struct EstimatedConfirmationFormatter {
    private let calendar: Calendar

    public init(locale: Locale = .current) {
        calendar = .current(locale: locale)
    }

    public func string(seconds: UInt32) -> String {
        let parts = DurationFormatter().estimateParts(seconds: Int64(seconds))
        guard
            parts.isEmpty == false,
            let duration = parts.string(style: .short, calendar: calendar, seconds: TimeInterval(seconds))
        else {
            return ""
        }
        return "≈ \(duration)"
    }
}

public struct CountdownFormatter {
    private let calendar: Calendar

    public init(locale: Locale = .current) {
        calendar = .current(locale: locale)
    }

    public func string(seconds: Int64) -> String? {
        let parts = DurationFormatter().countdownParts(seconds: seconds)
        guard parts.isEmpty == false else { return .none }
        return parts.string(style: .full, calendar: calendar, seconds: TimeInterval(seconds))
    }
}

private extension Calendar {
    static func current(locale: Locale) -> Calendar {
        var calendar = Calendar.current
        calendar.locale = locale
        return calendar
    }
}

private extension [GemDurationPart] {
    func string(style: DateComponentsFormatter.UnitsStyle, calendar: Calendar, seconds: TimeInterval) -> String? {
        let formatter = DateComponentsFormatter()
        formatter.allowedUnits = NSCalendar.Unit(map { $0.unit.calendarUnit })
        formatter.zeroFormattingBehavior = .dropAll
        formatter.unitsStyle = style
        formatter.calendar = calendar
        return formatter.string(from: seconds)
    }
}

private extension GemDurationUnit {
    var calendarUnit: NSCalendar.Unit {
        switch self {
        case .day: .day
        case .hour: .hour
        case .minute: .minute
        case .second: .second
        }
    }
}
