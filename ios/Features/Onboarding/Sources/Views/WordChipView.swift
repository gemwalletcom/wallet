// Copyright (c). Gem Wallet. All rights reserved.

import Style
import SwiftUI

struct WordChipView: View {
    private static let shakeOffsets: [CGFloat] = [6, 3].flatMap { [$0, -$0] } + [0]

    let word: String
    let isVerified: Bool
    let onPick: () -> Bool

    @State private var rejections = 0

    var body: some View {
        KeyframeAnimator(initialValue: CGFloat.zero, trigger: rejections) { offset in
            Button {
                if !onPick() {
                    rejections += 1
                }
            } label: {
                Text(word)
            }
            .buttonStyle(.styled(palette(offset: offset), paddingHorizontal: .small, paddingVertical: .tiny, glassEffect: .disabled))
            .disabled(isVerified)
            .fixedSize()
            .offset(x: offset)
            .accessibilityIdentifier("word_option")
        } keyframes: { _ in
            KeyframeTrack {
                for offset in Self.shakeOffsets {
                    CubicKeyframe(offset, duration: 0.07)
                }
            }
        }
        .sensoryFeedback(.error, trigger: rejections)
    }

    private func palette(offset: CGFloat) -> ButtonStylePalette {
        if offset != 0 {
            .red
        } else if isVerified {
            .lightGray
        } else {
            .blueGrayPressed
        }
    }
}
