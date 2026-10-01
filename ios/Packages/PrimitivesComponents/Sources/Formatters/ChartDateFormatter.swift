// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemCandleTick
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

    public func string(for date: Date, style: GemChartDateStyle) -> String {
        switch style {
        case .relative: TransactionDateFormatter(date: date, boundaries: .current(in: timeZone), locale: locale, timeZone: timeZone).row
        case .dayTime: date.formatted(dateTime.month(.abbreviated).day().hour().minute())
        case .day: date.formatted(dateTime.year().month(.abbreviated).day())
        }
    }

    public func string(for tick: GemCandleTick) -> String {
        switch tick.format {
        case .time: tick.date.formatted(dateTime.hour().minute())
        case .day: tick.date.formatted(dateTime.day().month(.abbreviated))
        case .monthYear: tick.date.formatted(dateTime.month(.abbreviated).year())
        }
    }

    private var dateTime: Date.FormatStyle {
        Date.FormatStyle(locale: locale, timeZone: timeZone)
    }
}
