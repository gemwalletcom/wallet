// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

struct ChartTimeAxis {
    enum Constants {
        static let maxLabels = 5.0
        static let edgeInset = 0.1
        static let day: TimeInterval = 86400
        static let yearSpan: TimeInterval = 360 * day
    }

    private enum Format {
        case time
        case timeAndDay
        case day
        case monthYear
    }

    let ticks: [Date]

    private let format: Format
    private let calendar: Calendar
    private let locale: Locale

    init(dates: [Date], range: ClosedRange<Date>, interval: TimeInterval, calendar: Calendar = .current, locale: Locale = .current) {
        let span = range.upperBound.timeIntervalSince(range.lowerBound)
        let start = range.lowerBound.addingTimeInterval(span * Constants.edgeInset)
        let visible = dates.filter { $0 >= start }
        let candles = Self.candlesPerLabel(visible.count, interval: interval)
        let first = dates.first ?? .distantPast
        format = Self.format(span: span, step: interval * Double(candles), covered: (dates.last ?? first).timeIntervalSince(first))
        self.calendar = calendar
        self.locale = locale
        ticks = stride(from: visible.count - 1, through: 0, by: -candles).map { visible[$0] }.reversed()
    }

    func label(for date: Date) -> String {
        switch format {
        case .time: date.formatted(timeStyle)
        case .timeAndDay: date.formatted(startsDay(date) ? dayStyle : timeStyle)
        case .day: date.formatted(dayStyle)
        case .monthYear: date.formatted(monthStyle)
        }
    }

    private static func candlesPerLabel(_ count: Int, interval: TimeInterval) -> Int {
        let gaps = Double(count - 1)
        let rough = gaps / (Constants.maxLabels - 1)
        let fitting = max(1, rough.rounded(.down))
        let candles = Int(gaps / fitting < Constants.maxLabels ? fitting : max(1, rough.rounded(.up)))
        guard interval > 0, interval < Constants.day, Double(candles) * interval >= Constants.day else {
            return candles
        }
        let candlesPerDay = Int((Constants.day / interval).rounded())
        return Int((Double(candles) / Double(candlesPerDay)).rounded(.up)) * candlesPerDay
    }

    private static func format(span: TimeInterval, step: TimeInterval, covered: TimeInterval) -> Format {
        if span >= Constants.yearSpan {
            .monthYear
        } else if step >= Constants.day {
            .day
        } else if covered <= Constants.day {
            .time
        } else {
            .timeAndDay
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
