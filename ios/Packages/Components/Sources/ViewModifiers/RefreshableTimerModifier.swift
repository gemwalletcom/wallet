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

    @State private var refreshedAt = ContinuousClock.now
    @State private var isRefreshing = false

    func body(content: Content) -> some View {
        let run = TimerRun(isRunning: scenePhase == .active || isRefreshing, interval: interval)
        content
            .refreshable { await refresh(.user) }
            .task(id: run) {
                guard run.isRunning, run.interval > 0 else { return }
                while !Task.isCancelled {
                    let start = isRefreshing || scenePhase != .active ? ContinuousClock.now : refreshedAt
                    try? await Task.sleep(until: start.advanced(by: .seconds(interval)), clock: .continuous)
                    guard !Task.isCancelled else { return }
                    if scenePhase == .active, refreshedAt.advanced(by: .seconds(interval)) <= .now {
                        await refresh(.timer)
                    }
                }
            }
    }

    private func refresh(_ source: RefreshSource) async {
        guard !isRefreshing else { return }
        refreshedAt = .now
        isRefreshing = true
        await action(source)
        isRefreshing = false
    }
}

private struct TimerRun: Equatable {
    let isRunning: Bool
    let interval: TimeInterval
}

public extension View {
    func refreshableTimer(every interval: TimeInterval, action: @Sendable @escaping (RefreshSource) async -> Void) -> some View {
        modifier(RefreshableTimerModifier(interval: interval, action: action))
    }
}
