// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Style
import SwiftUI

public struct ListItemExpiryView: View {
    private let title: String
    private let expiresAt: Date

    public init(title: String, expiresAt: Date) {
        self.title = title
        self.expiresAt = expiresAt
    }

    public var body: some View {
        HStack {
            Text(title)
                .textStyle(.body)
            Spacer()
            Text(timerInterval: countdown, countsDown: true)
                .multilineTextAlignment(.trailing)
                .monospacedDigit()
                .textStyle(.bodySecondary)
        }
    }
}

// MARK: - Private

extension ListItemExpiryView {
    private var countdown: ClosedRange<Date> {
        let now = Date.now
        return now ... max(expiresAt, now)
    }
}

// MARK: - Previews

#Preview {
    List {
        ListItemExpiryView(title: "Payment expires in", expiresAt: .now.addingTimeInterval(90))
        ListItemExpiryView(title: "Payment expires in", expiresAt: .now.addingTimeInterval(-90))
    }
}
