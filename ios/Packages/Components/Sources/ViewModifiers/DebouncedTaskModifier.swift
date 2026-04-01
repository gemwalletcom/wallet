// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

private struct DebouncedTaskModifier<T: DebouncableTrigger>: ViewModifier {
    @State private var task: Task<Void, Never>?

    let trigger: T?
    let interval: Duration
    let action: @Sendable () async -> Void

    func body(content: Content) -> some View {
        content
            .onAppear {
                restartTask(for: trigger)
            }
            .onChange(of: trigger) { _, newTrigger in
                restartTask(for: newTrigger)
            }
            .onDisappear {
                task?.cancel()
                task = nil
            }
    }

    private func restartTask(for trigger: T?) {
        task?.cancel()
        guard let trigger else { return }

        task = Task {
            if !trigger.isImmediate {
                try? await Task.sleep(for: interval)
                guard !Task.isCancelled else { return }
            }
            await action()
        }
    }
}

public extension View {
    func debouncedTask(
        id trigger: (some DebouncableTrigger)?,
        interval: Duration = .debounce,
        action: @Sendable @escaping () async -> Void,
    ) -> some View {
        modifier(DebouncedTaskModifier(trigger: trigger, interval: interval, action: action))
    }
}
