// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
@testable import PrimitivesComponents
import Testing

struct TransactionDateFormatterTests {
    private let locale = Locale.US
    private let timeZone = TimeZone.NewYork!
    private let calendar: Calendar

    init() {
        var calendar = Calendar.current
        calendar.locale = locale
        calendar.timeZone = timeZone
        self.calendar = calendar
    }

    private func formatter(for date: Date) -> TransactionDateFormatter {
        TransactionDateFormatter(date: date, boundaries: .current(in: timeZone), locale: locale, timeZone: timeZone)
    }

    @Test
    func aRowNamesTodayAndYesterdayBeforeItDatesThem() throws {
        let today = try #require(calendar.date(bySettingHour: 14, minute: 30, second: 0, of: Date()))
        let yesterdayDate = try #require(calendar.date(byAdding: .day, value: -1, to: Date()))
        let yesterday = try #require(calendar.date(bySettingHour: 9, minute: 15, second: 0, of: yesterdayDate))
        let older = try #require(calendar.date(from: DateComponents(year: 2025, month: 2, day: 2, hour: 10, minute: 25)))

        #expect(formatter(for: today).row == "Today, 2:30\u{202F}PM")
        #expect(formatter(for: yesterday).row == "Yesterday, 9:15\u{202F}AM")
        #expect(formatter(for: older).row == "February 2, 2025 at 10:25\u{202F}AM")
    }

    @Test
    func aSectionNamesTheDayAlone() throws {
        let today = try #require(calendar.date(bySettingHour: 14, minute: 30, second: 0, of: Date()))
        let older = try #require(calendar.date(from: DateComponents(year: 2025, month: 2, day: 2, hour: 10, minute: 25)))

        #expect(formatter(for: today).section == "Today")
        #expect(formatter(for: older).section == "February 2, 2025")
        #expect(formatter(for: older).day == "Feb 2, 2025")
    }

    @Test
    func aChangedLocaleOrClockFormatsAfresh() throws {
        let morning = try #require(calendar.date(from: DateComponents(year: 2025, month: 2, day: 2, hour: 7, minute: 5)))
        var twentyFourHour = Locale.Components(locale: locale)
        twentyFourHour.hourCycle = .zeroToTwentyThree
        let row = { (locale: Locale) in
            TransactionDateFormatter(date: morning, boundaries: .current(in: timeZone), locale: locale, timeZone: timeZone).row
        }

        #expect(row(locale) == "February 2, 2025 at 7:05\u{202F}AM")
        #expect(row(Locale(components: twentyFourHour)) == "February 2, 2025 at 07:05")
        #expect(row(Locale(identifier: "en_GB")) == "2 February 2025 at 07:05")
    }

    @Test
    func aMissingTimestampReadsAsNothing() {
        let formatter = TransactionDateFormatter(unixMilliseconds: 0, locale: locale, timeZone: timeZone)

        #expect(formatter.row.isEmpty)
        #expect(formatter.section.isEmpty)
        #expect(formatter.day.isEmpty)
    }
}
