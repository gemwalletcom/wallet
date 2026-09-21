// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import func Gemstone.chartDateStyle
import enum Gemstone.GemChartDateStyle
import GemstonePrimitives
import Primitives

public struct ChartDateFormatter: Sendable {
    private let locale: Locale
    private let timeZone: TimeZone

    public init(
        locale: Locale = .current,
        timeZone: TimeZone = .current,
    ) {
        self.locale = locale
        self.timeZone = timeZone
    }

    public func string(for date: Date, period: ChartPeriod) -> String {
        switch chartDateStyle(period: period.toGem()) {
        case .relative: TransactionDateFormatter(date: date, boundaries: .current(in: timeZone), locale: locale, timeZone: timeZone).row
        case .dayTime: date.formatted(dateTime.month(.abbreviated).day().hour().minute())
        case .day: date.formatted(dateTime.year().month(.abbreviated).day())
        }
    }

    private var dateTime: Date.FormatStyle {
        Date.FormatStyle(locale: locale, timeZone: timeZone)
    }
}
