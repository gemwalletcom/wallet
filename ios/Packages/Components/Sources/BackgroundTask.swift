// Copyright (c). Gem Wallet. All rights reserved.

import UIKit

@MainActor
public func withBackgroundTask<T>(_ operation: @MainActor () async throws -> T) async rethrows -> T {
    let task = BackgroundTaskHandle()
    defer { task.end() }
    return try await operation()
}

@MainActor
private final class BackgroundTaskHandle {
    private var identifier = UIBackgroundTaskIdentifier.invalid

    init() {
        identifier = UIApplication.shared.beginBackgroundTask { [weak self] in
            self?.end()
        }
    }

    func end() {
        guard identifier != .invalid else { return }
        UIApplication.shared.endBackgroundTask(identifier)
        identifier = .invalid
    }
}
