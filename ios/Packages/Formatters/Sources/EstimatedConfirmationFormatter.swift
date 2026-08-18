// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public struct EstimatedConfirmationFormatter {
    private let calendar: Calendar

    public init(locale: Locale = .current) {
        var calendar = Calendar.current
        calendar.locale = locale
        self.calendar = calendar
    }

    public func string(seconds: UInt32) -> String {
        let minutes = max(1, (Double(seconds) / 60).rounded())
        let formatter = DateComponentsFormatter()
        formatter.allowedUnits = [.minute]
        formatter.unitsStyle = .short
        formatter.calendar = calendar
        let duration = formatter.string(from: minutes * 60) ?? "\(Int(minutes)) min"
        return "≈ \(duration)"
    }
}
