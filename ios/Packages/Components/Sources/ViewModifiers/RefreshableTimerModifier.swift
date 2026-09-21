// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

public enum RefreshSource: Sendable {
    case timer
    case user
}

private struct RefreshableTimerModifier: ViewModifier {
    @Environment(\.scenePhase) private var scenePhase

    let interval: TimeInterval
    let action: @Sendable (RefreshSource) async -> Void

    @State private var trigger = 0
    @State private var refreshedAt = ContinuousClock.now
    @State private var isRefreshing = false

    func body(content: Content) -> some View {
        content
            .refreshable { await refresh(.user) }
            .onChange(of: scenePhase) { _, phase in
                if phase == .active, !isRefreshing {
                    trigger += 1
                }
            }
            .task(id: TimerRun(trigger: trigger, interval: interval)) {
                guard interval > 0 else { return }
                while !Task.isCancelled, scenePhase == .active {
                    let start = isRefreshing ? ContinuousClock.now : refreshedAt
                    try? await Task.sleep(until: start.advanced(by: .seconds(interval)), clock: .continuous)
                    guard !Task.isCancelled, scenePhase == .active else { return }
                    await refresh(.timer)
                }
            }
    }

    private func refresh(_ source: RefreshSource) async {
        guard !isRefreshing else { return }
        refreshedAt = .now
        isRefreshing = true
        await action(source)
        isRefreshing = false
        if case .user = source {
            trigger += 1
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
