// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Style
import SwiftUI

struct SupportTypingIndicator: View {
    let name: String

    var body: some View {
        VStack(alignment: .leading, spacing: .tiny) {
            Text(name)
                .font(.caption)
                .foregroundStyle(Colors.secondaryText)
            HStack(spacing: .zero) {
                SupportTypingDots()
                    .padding(.horizontal, .space16)
                    .padding(.vertical, .space12)
                    .background(Colors.white)
                    .clipShape(Capsule())
                Spacer(minLength: .space32)
            }
        }
    }
}

private struct SupportTypingDots: View {
    private enum Constants {
        static let dotCount = 3
    }

    @State private var animating = false

    var body: some View {
        HStack(spacing: .tiny) {
            ForEach(0 ..< Constants.dotCount, id: \.self) { index in
                Circle()
                    .fill(Colors.secondaryText)
                    .frame(size: .space8)
                    .opacity(animating ? 1 : .medium)
                    .animation(
                        .easeInOut(duration: Interval.AnimationDuration.slow)
                            .repeatForever()
                            .delay(Double(index) * Interval.AnimationDuration.normal),
                        value: animating,
                    )
            }
        }
        .onAppear { animating = true }
    }
}
