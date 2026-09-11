// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public struct EstimatedConfirmationFormatter {
    private let calendar: Calendar

    public init(locale: Locale = .current) {
        var calendar = Calendar.current
        calendar.locale = locale
        self.calendar = calendar
    }

    public func string(minutes: UInt32) -> String {
        let formatter = DateComponentsFormatter()
        formatter.allowedUnits = [.minute]
        formatter.unitsStyle = .short
        formatter.calendar = calendar
        let duration = formatter.string(from: TimeInterval(minutes) * 60) ?? "\(minutes) min"
        return "≈ \(duration)"
    }
}
