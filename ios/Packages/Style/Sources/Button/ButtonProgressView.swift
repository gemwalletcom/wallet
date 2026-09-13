// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

struct ButtonProgressView: View {
    private static let rotationDuration: TimeInterval = 1
    private static let arc: CGFloat = 0.75

    var body: some View {
        TimelineView(.animation) { context in
            Circle()
                .trim(from: 0, to: Self.arc)
                .stroke(Colors.whiteSolid, style: StrokeStyle(lineWidth: .space2, lineCap: .round))
                .padding(.space1)
                .frame(width: Sizing.image.small, height: Sizing.image.small)
                .rotationEffect(.degrees(angle(at: context.date)))
        }
    }

    private func angle(at date: Date) -> Double {
        let progress = date.timeIntervalSinceReferenceDate.truncatingRemainder(dividingBy: Self.rotationDuration)
        return progress / Self.rotationDuration * 360
    }
}
