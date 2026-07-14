// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import SwiftUI

@Observable
@MainActor
public final class ConnectionStatusObserver {
    private let monitors: [any ConnectionComponentMonitoring]

    public private(set) var healthByComponent: [ConnectionComponent: ConnectionComponentHealth] = [:]

    @ObservationIgnored private var tasks: [Task<Void, Never>] = []

    public nonisolated init(monitors: [any ConnectionComponentMonitoring] = [InternetConnectionMonitor()]) {
        self.monitors = monitors
    }

    public func start() {
        guard tasks.isEmpty else { return }
        tasks = monitors.map { monitor in
            Task { [weak self] in
                for await health in monitor.healthStream() {
                    self?.update(component: monitor.component, health: health)
                }
            }
        }
    }

    public func stop() {
        tasks.forEach { $0.cancel() }
        tasks = []
    }

    public var status: ConnectionStatus {
        healthByComponent
            .filter { $0.value.isHealthy == false }
            .keys
            .map(\.failureStatus)
            .max { $0.severity < $1.severity } ?? .online
    }

    func update(component: ConnectionComponent, health: ConnectionComponentHealth) {
        if component == .internet, health.isHealthy, healthByComponent[.internet]?.isHealthy == false {
            healthByComponent = [:]
        }
        healthByComponent[component] = health
    }
}

// MARK: - EnvironmentValues

public extension EnvironmentValues {
    @Entry var connectionStatus = ConnectionStatusObserver()
}
