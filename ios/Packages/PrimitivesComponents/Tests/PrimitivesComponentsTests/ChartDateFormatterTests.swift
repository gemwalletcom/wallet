// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import Foundation
import Primitives
@testable import PrimitivesComponents
import Testing

struct ChartDateFormatterTests {
    @Test
    func stringForPeriod() throws {
        let timeZone = TimeZone.NewYork!
        let today = try #require(Calendar.current.date(bySettingHour: 14, minute: 30, second: 0, of: Date()))
        let fixed = Date(timeIntervalSince1970: 1_745_505_000)

        let us = ChartDateFormatter(
            relative: RelativeDateFormatter(locale: .US, timeZone: timeZone),
            locale: .US,
            timeZone: timeZone,
        )
        #expect(us.string(for: today, period: .day) == "Today, 2:30 PM")
        #expect(us.string(for: fixed, period: .hour) == "9:30 AM")
        #expect(us.string(for: fixed, period: .week) == "Apr 24, 9:30 AM")
        #expect(us.string(for: fixed, period: .month) == "Apr 24, 9:30 AM")
        #expect(us.string(for: fixed, period: .year) == "Apr 24, 2025")
        #expect(us.string(for: fixed, period: .all) == "Apr 24, 2025")

        let de = ChartDateFormatter(
            relative: RelativeDateFormatter(locale: Locale(identifier: "de_DE"), timeZone: timeZone),
            locale: Locale(identifier: "de_DE"),
            timeZone: timeZone,
        )
        #expect(de.string(for: today, period: .day) == "Heute, 14:30")
        #expect(de.string(for: fixed, period: .hour) == "09:30")
        #expect(de.string(for: fixed, period: .year) == "24. Apr. 2025")
    }
}
