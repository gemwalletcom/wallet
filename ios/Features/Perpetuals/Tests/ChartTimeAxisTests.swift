// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import Perpetuals
import Testing

struct ChartTimeAxisTests {
    @Test
    func anHourIsLabelledOnItsCandlesBackFromTheNewest() throws {
        let axis = try axis(from: "2026-09-23T15:08:00Z", every: 60, count: 61)

        #expect(labels(axis) == ["15:16", "15:29", "15:42", "15:55", "16:08"])
    }

    @Test
    func aChartWithinADayShowsTimesOnly() throws {
        #expect(try labels(axis(from: "2026-09-22T16:30:00Z", every: 1800, count: 48)) == ["20:00", "01:00", "06:00", "11:00", "16:00"])
        #expect(try labels(axis(from: "2026-09-22T20:00:00Z", every: 3600, count: 21)) == ["00:00", "04:00", "08:00", "12:00", "16:00"], "midnight stays a time")
    }

    @Test
    func aChartOverMoreThanADayShowsTheDateOnTheFirstLabelOfAnEarlierDay() throws {
        #expect(try labels(axis(from: "2026-09-22T04:00:00Z", every: 4 * 3600, count: 9)) == ["22 Sep", "20:00", "04:00", "12:00"])
        #expect(try labels(axis(from: "2026-09-21T12:00:00Z", every: 4 * 3600, count: 13)) == ["22 Sep", "12:00", "23 Sep", "12:00"], "a label at midnight names the day")
    }

    @Test
    func dailyStepsLandOnWholeDaysOfCandles() throws {
        let axis = try axis(from: "2026-09-16T16:00:00Z", every: 4 * 3600, count: 43)

        #expect(labels(axis) == ["17 Sep", "19 Sep", "21 Sep", "23 Sep"])
    }

    @Test
    func aChartOverAYearShowsTheYear() throws {
        let first = try date("2025-08-07T00:00:00Z")
        let dates = (0 ..< 14).compactMap { Self.calendar.date(byAdding: .month, value: $0, to: first) }
        let axis = ChartTimeAxis(dates: dates, range: viewport(dates, interval: 28 * 86400), interval: 28 * 86400, calendar: Self.calendar, locale: Self.locale)

        #expect(labels(axis) == ["Sep 2025", "Dec 2025", "Mar 2026", "Jun 2026", "Sep 2026"])
    }

    @Test
    func everyLabelIsACandleEndingOnTheNewest() throws {
        let series: [(TimeInterval, Int)] = [(60, 1), (60, 60), (1800, 48), (4 * 3600, 42), (12 * 3600, 60), (7 * 86400, 52), (30 * 86400, 40)]

        for (interval, count) in series {
            let first = try date("2026-01-05T00:00:00Z")
            let dates = (0 ..< count).map { first.addingTimeInterval(Double($0) * interval) }
            let axis = ChartTimeAxis(dates: dates, range: viewport(dates, interval: interval), interval: interval, calendar: Self.calendar, locale: Self.locale)

            #expect(axis.ticks.allSatisfy(dates.contains), "\(count) candles of \(interval) s")
            #expect(axis.ticks.last == dates.last, "\(count) candles of \(interval) s")
            #expect(axis.ticks.count <= 5, "\(count) candles of \(interval) s")
        }
    }
}

extension ChartTimeAxisTests {
    private static let locale = Locale(identifier: "en_GB")

    private static let calendar: Calendar = {
        var calendar = Calendar(identifier: .gregorian)
        calendar.timeZone = .gmt
        calendar.locale = locale
        return calendar
    }()

    private func date(_ value: String) throws -> Date {
        try Date(value, strategy: .iso8601)
    }

    private func viewport(_ dates: [Date], interval: TimeInterval) -> ClosedRange<Date> {
        let first = dates.first ?? .distantPast
        let last = dates.last ?? first
        let room = max(last.timeIntervalSince(first) * 0.1, interval / 2)
        return first.addingTimeInterval(-interval / 2) ... last.addingTimeInterval(room)
    }

    private func axis(from start: String, every interval: TimeInterval, count: Int) throws -> ChartTimeAxis {
        let first = try date(start)
        let dates = (0 ..< count).map { first.addingTimeInterval(Double($0) * interval) }
        return ChartTimeAxis(dates: dates, range: viewport(dates, interval: interval), interval: interval, calendar: Self.calendar, locale: Self.locale)
    }

    private func labels(_ axis: ChartTimeAxis) -> [String] {
        axis.ticks.map(axis.label(for:))
    }
}
