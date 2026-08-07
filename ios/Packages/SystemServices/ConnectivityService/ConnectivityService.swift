// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public actor ConnectivityService {
    private static let offlineDebounce: Duration = .milliseconds(500)

    private let monitor: any ConnectivityMonitoring
    
    private var state: ConnectivityState = .unknown

    private var subscribers: [UUID: AsyncStream<ConnectivityState>.Continuation] = [:]
    private var monitorTask: Task<Void, Never>?
    private var offlineTask: Task<Void, Never>?

    public init(monitor: any ConnectivityMonitoring = ConnectivityMonitor()) {
        self.monitor = monitor
    }

    deinit {
        monitorTask?.cancel()
        offlineTask?.cancel()
        subscribers.values.forEach { $0.finish() }
    }
    
    public var status: ConnectivityState { state }

    public func start() {
        monitorTask?.cancel()
        let states = monitor.stateStream()
        monitorTask = Task { [weak self] in
            for await state in states {
                await self?.apply(state)
            }
        }
    }

    public func observe() -> AsyncStream<ConnectivityState> {
        let (stream, continuation) = AsyncStream.makeStream(
            of: ConnectivityState.self,
            bufferingPolicy: .bufferingNewest(1),
        )
        let id = UUID()
        subscribers[id] = continuation
        continuation.yield(state)
        continuation.onTermination = { [weak self] _ in
            Task { await self?.removeSubscriber(id) }
        }
        return stream
    }

    private func apply(_ state: ConnectivityState) {
        offlineTask?.cancel()
        offlineTask = nil

        guard state.isOffline, !self.state.isOffline else {
            updateState(state)
            return
        }
        offlineTask = Task { [weak self] in
            do {
                try await Task.sleep(for: ConnectivityService.offlineDebounce)
                await self?.updateState(state)
            } catch {}
        }
    }

    private func updateState(_ state: ConnectivityState) {
        guard state != self.state else { return }
        self.state = state
        notifySubscribers()
    }

    private func notifySubscribers() {
        for subscriber in subscribers.values {
            subscriber.yield(state)
        }
    }

    private func removeSubscriber(_ id: UUID) {
        subscribers[id] = nil
    }
}
