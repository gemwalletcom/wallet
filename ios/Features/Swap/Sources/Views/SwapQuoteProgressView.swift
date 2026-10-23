// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemSwapQuoteProgress
import Style
import SwiftUI

struct SwapQuoteProgressView: View {
    let progress: GemSwapQuoteProgress

    var body: some View {
        HStack(spacing: .tiny) {
            ZStack {
                Circle()
                    .stroke(Colors.grayVeryLight, lineWidth: .space2)
                Circle()
                    .trim(from: 0, to: progress.fraction)
                    .stroke(Colors.gray, style: StrokeStyle(lineWidth: .space2, lineCap: .round))
                    .rotationEffect(.degrees(-90))
            }
            .frame(width: Sizing.image.tiny, height: Sizing.image.tiny)
            Text(progress.text)
                .textStyle(TextStyle(font: .footnote, color: Colors.gray, fontWeight: .medium))
                .numericTransition(for: progress.text)
        }
        .animation(.default, value: progress.fraction)
    }
}
