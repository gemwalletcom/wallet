// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

public enum RefreshSource: Sendable {
    case timer
    case user
}

private struct RefreshableTimerModifier: ViewModifier {
    let interval: TimeInterval
    let action: @Sendable (RefreshSource) async -> Void

    @State private var trigger = 0

    func body(content: Content) -> some View {
        content
            .refreshable {
                trigger += 1
                await action(.user)
            }
            .task(id: TimerRun(trigger: trigger, interval: interval)) {
                while !Task.isCancelled {
                    try? await Task.sleep(for: .seconds(interval))
                    guard !Task.isCancelled else { break }
                    await action(.timer)
                }
            }
    }
}

private struct TimerRun: Equatable {
    let trigger: Int
    let interval: TimeInterval
}

public extension View {
    func refreshableTimer(every interval: TimeInterval, action: @Sendable @escaping (RefreshSource) async -> Void) -> some View {
        modifier(RefreshableTimerModifier(interval: interval, action: action))
    }
}
