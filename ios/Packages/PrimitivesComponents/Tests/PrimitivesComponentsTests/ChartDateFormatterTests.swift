// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import Foundation
import Primitives
@testable import PrimitivesComponents
import Testing

struct ChartDateFormatterTests {
    private let locale = Locale.US
    private let timeZone = TimeZone.NewYork!
    private let formatter: ChartDateFormatter
    private let calendar: Calendar

    init() {
        let relative = RelativeDateFormatter(locale: locale, timeZone: timeZone)
        formatter = ChartDateFormatter(relative: relative, locale: locale, timeZone: timeZone)
        calendar = relative.calendar
    }

    @Test
    func stringForPeriod() throws {
        let today = try #require(calendar.date(bySettingHour: 14, minute: 30, second: 0, of: Date()))
        let fixed = try #require(calendar.date(from: DateComponents(year: 2025, month: 4, day: 24, hour: 14, minute: 30)))

        #expect(formatter.string(for: today, period: .hour) == "Today, 2:30\u{202F}PM")
        #expect(formatter.string(for: today, period: .day) == "Today, 2:30\u{202F}PM")
        #expect(formatter.string(for: fixed, period: .week) == "Apr 24 at 2:30\u{202F}PM")
        #expect(formatter.string(for: fixed, period: .month) == "Apr 24 at 2:30\u{202F}PM")
        #expect(formatter.string(for: fixed, period: .year) == "Apr 24, 2025")
        #expect(formatter.string(for: fixed, period: .all) == "Apr 24, 2025")
    }
}
