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
        let formatter = DateComponentsFormatter()
        formatter.allowedUnits = [.minute, .second]
        formatter.zeroFormattingBehavior = .dropAll
        formatter.unitsStyle = .short
        formatter.calendar = calendar
        guard let duration = formatter.string(from: TimeInterval(seconds)) else {
            return .empty
        }
        return "≈ \(duration)"
    }
}
