// Copyright (c). Gem Wallet. All rights reserved.

import Style
import SwiftUI

struct PulsingDotView: View {
    private let color: Color
    private let dotSize: CGFloat = 8

    init(color: Color) {
        self.color = color
    }

    var body: some View {
        ZStack {
            pulseRing(scale: 3.0, delay: 0)
            pulseRing(scale: 2.5, delay: 0.4)
            centerDot
        }
    }
}

// MARK: - Components

extension PulsingDotView {
    private var centerDot: some View {
        Circle()
            .fill(
                RadialGradient(
                    colors: [Colors.white, color],
                    center: .center,
                    startRadius: 0,
                    endRadius: dotSize / 2,
                ),
            )
            .frame(width: dotSize, height: dotSize)
            .shadow(color: color.opacity(.strong), radius: 6)
            .shadow(color: color.opacity(.semiMedium), radius: 12)
    }

    private func pulseRing(scale: CGFloat, delay: Double) -> some View {
        PulseRingView(
            color: color,
            size: dotSize,
            maxScale: scale,
            delay: delay,
        )
    }
}

// MARK: - PulseRingView

private struct PulseRingView: View {
    let color: Color
    let size: CGFloat
    let maxScale: CGFloat
    let delay: Double

    private let duration: Double = .AnimationDuration.verySlow

    var body: some View {
        TimelineView(.animation) { timeline in
            let progress = progress(at: timeline.date)
            Circle()
                .stroke(color.opacity(.semiMedium), lineWidth: 1.5)
                .frame(width: size, height: size)
                .scaleEffect(1.0 + (maxScale - 1.0) * progress)
                .opacity(0.8 * (1.0 - progress))
        }
    }

    private func progress(at date: Date) -> Double {
        let elapsed = date.timeIntervalSinceReferenceDate + delay
        let normalized = elapsed.truncatingRemainder(dividingBy: duration) / duration
        return 1.0 - pow(1.0 - normalized, 2)
    }
}
