// Copyright (c). Gem Wallet. All rights reserved.

import QuartzCore
import SwiftUI

@MainActor
private final class ChartRangeFollower: NSObject {
    private enum Constants {
        static let growSpring = Spring(mass: 1, stiffness: 600, damping: 2 * 600.squareRoot())
        static let shrinkSpring = Spring(mass: 1, stiffness: 200, damping: 2 * 200.squareRoot())
        static let settleFraction: Double = 0.005
    }

    private var range = Binding.constant(0.0 ... 1.0)
    private var target = 0.0 ... 1.0
    private var lowerVelocity: Double = 0
    private var upperVelocity: Double = 0
    private var lastFrameTime: CFTimeInterval = 0
    private var link: CADisplayLink?

    func follow(_ target: ClosedRange<Double>, in range: Binding<ClosedRange<Double>>) {
        self.target = target
        self.range = range
        guard link == nil else { return }
        guard !isSettled(range.wrappedValue) else {
            range.wrappedValue = target
            return
        }
        lastFrameTime = CACurrentMediaTime()
        let link = CADisplayLink(target: self, selector: #selector(step))
        link.add(to: .main, forMode: .common)
        self.link = link
    }

    func stop() {
        link?.invalidate()
        link = nil
        lowerVelocity = 0
        upperVelocity = 0
    }

    @objc private func step(_ link: CADisplayLink) {
        let seconds = link.targetTimestamp - lastFrameTime
        lastFrameTime = link.targetTimestamp
        var lower = range.wrappedValue.lowerBound
        var upper = range.wrappedValue.upperBound
        spring(growing: target.lowerBound < lower).update(value: &lower, velocity: &lowerVelocity, target: target.lowerBound, deltaTime: seconds)
        spring(growing: target.upperBound > upper).update(value: &upper, velocity: &upperVelocity, target: target.upperBound, deltaTime: seconds)
        let next = min(lower, upper) ... max(lower, upper)
        guard !isSettled(next) else {
            range.wrappedValue = target
            return stop()
        }
        range.wrappedValue = next
    }

    private func isSettled(_ range: ClosedRange<Double>) -> Bool {
        let tolerance = (target.upperBound - target.lowerBound) * Constants.settleFraction
        return abs(range.lowerBound - target.lowerBound) <= tolerance && abs(range.upperBound - target.upperBound) <= tolerance
    }

    private func spring(growing: Bool) -> Spring {
        growing ? Constants.growSpring : Constants.shrinkSpring
    }
}

private struct ChartRangeModifier: ViewModifier {
    @Binding var range: ClosedRange<Double>
    let target: ClosedRange<Double>

    @State private var follower = ChartRangeFollower()

    func body(content: Content) -> some View {
        content
            .onAppear {
                follower.follow(target, in: $range)
            }
            .onChange(of: target) { _, target in
                follower.follow(target, in: $range)
            }
            .onDisappear {
                follower.stop()
            }
    }
}

// MARK: - View Modifier

public extension View {
    func chartRange(_ range: Binding<ClosedRange<Double>>, fitting target: ClosedRange<Double>) -> some View {
        modifier(ChartRangeModifier(range: range, target: target))
    }
}
