// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemCandleTickFormat
@testable import Perpetuals
import Testing

struct ChartTimeAxisTests {
    @Test
    func eachFormatNamesItsTicks() throws {
        #expect(try labels(["2026-09-23T15:16:00Z", "2026-09-23T16:08:00Z"], .time) == ["15:16", "16:08"])
        #expect(try labels(["2026-09-17T16:00:00Z", "2026-09-19T16:00:00Z"], .day) == ["17 Sep", "19 Sep"])
        #expect(try labels(["2025-09-07T00:00:00Z", "2026-09-07T00:00:00Z"], .monthYear) == ["Sep 2025", "Sep 2026"])
    }

    @Test
    func timesStayTimesOnAChartWithinADay() throws {
        #expect(try labels(["2026-09-22T20:00:00Z", "2026-09-23T00:00:00Z", "2026-09-23T04:00:00Z"], .time) == ["20:00", "00:00", "04:00"], "midnight stays a time")
    }

    @Test
    func aLongerChartDatesTheFirstLabelOfAnEarlierDay() throws {
        #expect(try labels(["2026-09-22T12:00:00Z", "2026-09-22T20:00:00Z", "2026-09-23T04:00:00Z", "2026-09-23T12:00:00Z"], .timeOrDay) == ["22 Sep", "20:00", "04:00", "12:00"])
        #expect(try labels(["2026-09-22T00:00:00Z", "2026-09-22T12:00:00Z", "2026-09-23T00:00:00Z", "2026-09-23T12:00:00Z"], .timeOrDay) == ["22 Sep", "12:00", "23 Sep", "12:00"], "a label at midnight names the day")
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

    private func labels(_ ticks: [String], _ format: GemCandleTickFormat) throws -> [String] {
        let axis = try ChartTimeAxis(ticks: ticks.map { try Date($0, strategy: .iso8601) }, format: format, calendar: Self.calendar, locale: Self.locale)
        return axis.ticks.map(axis.label(for:))
    }
}
