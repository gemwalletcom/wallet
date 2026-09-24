// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemCandleTickFormat

struct ChartTimeAxis {
    let ticks: [Date]

    private let format: GemCandleTickFormat
    private let calendar: Calendar
    private let locale: Locale

    init(ticks: [Date], format: GemCandleTickFormat, calendar: Calendar = .current, locale: Locale = .current) {
        self.ticks = ticks
        self.format = format
        self.calendar = calendar
        self.locale = locale
    }

    func label(for date: Date) -> String {
        switch format {
        case .time: date.formatted(timeStyle)
        case .timeOrDay: date.formatted(startsDay(date) ? dayStyle : timeStyle)
        case .day: date.formatted(dayStyle)
        case .monthYear: date.formatted(monthStyle)
        }
    }
}

// MARK: - Private

private extension ChartTimeAxis {
    var style: Date.FormatStyle {
        Date.FormatStyle(locale: locale, calendar: calendar, timeZone: calendar.timeZone)
    }

    var timeStyle: Date.FormatStyle {
        style.hour().minute()
    }

    var dayStyle: Date.FormatStyle {
        style.day().month(.abbreviated)
    }

    var monthStyle: Date.FormatStyle {
        style.month(.abbreviated).year()
    }

    func startsDay(_ date: Date) -> Bool {
        guard let latest = ticks.last else { return false }
        let isFirstOfEarlierDay = !calendar.isDate(date, inSameDayAs: latest) && ticks.first { calendar.isDate($0, inSameDayAs: date) } == date
        return isFirstOfEarlierDay || calendar.startOfDay(for: date) == date
    }
}
