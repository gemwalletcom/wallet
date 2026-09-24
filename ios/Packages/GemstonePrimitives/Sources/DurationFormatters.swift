// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import func Gemstone.estimatedDurationParts
import struct Gemstone.GemDurationPart
import enum Gemstone.GemDurationUnit

public struct EstimatedConfirmationFormatter {
    private let calendar: Calendar

    public init(locale: Locale = .current) {
        calendar = .current(locale: locale)
    }

    public func string(seconds: UInt32) -> String {
        string(parts: estimatedDurationParts(seconds: Int64(seconds)))
    }

    public func string(parts: [GemDurationPart]) -> String {
        guard let duration = parts.string(style: .short, calendar: calendar) else {
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

    public func string(parts: [GemDurationPart]) -> String? {
        parts.string(style: .full, calendar: calendar)
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
    func string(style: DateComponentsFormatter.UnitsStyle, calendar: Calendar) -> String? {
        guard isEmpty == false else { return nil }
        let formatter = DateComponentsFormatter()
        formatter.allowedUnits = NSCalendar.Unit(map(\.unit.calendarUnit))
        formatter.zeroFormattingBehavior = .dropAll
        formatter.unitsStyle = style
        formatter.calendar = calendar
        return formatter.string(from: components)
    }

    var components: DateComponents {
        reduce(into: DateComponents()) { components, part in
            switch part.unit {
            case .day: components.day = Int(part.value)
            case .hour: components.hour = Int(part.value)
            case .minute: components.minute = Int(part.value)
            case .second: components.second = Int(part.value)
            }
        }
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
