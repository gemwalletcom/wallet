// Copyright (c). Gem Wallet. All rights reserved.

import os

public final class SystemPrompt: Sendable {
    private struct State {
        var presented = 0
        var endedAt: ContinuousClock.Instant?
    }

    private let state = OSAllocatedUnfairLock(initialState: State())

    public init() {}

    public func hasPresented(since instant: ContinuousClock.Instant) -> Bool {
        state.withLock { state in
            guard state.presented == 0 else { return true }
            guard let endedAt = state.endedAt else { return false }
            return endedAt > instant
        }
    }

    @discardableResult
    public func presenting<T>(_ work: () throws -> T) rethrows -> T {
        begin()
        defer { end() }
        return try work()
    }

    @discardableResult
    public func presenting<T>(isolation _: isolated (any Actor)? = #isolation, _ work: () async throws -> T) async rethrows -> T {
        begin()
        defer { end() }
        return try await work()
    }
}

// MARK: - Private

extension SystemPrompt {
    private func begin() {
        state.withLock { $0.presented += 1 }
    }

    private func end() {
        state.withLock { state in
            state.presented -= 1
            state.endedAt = .now
        }
    }
}
