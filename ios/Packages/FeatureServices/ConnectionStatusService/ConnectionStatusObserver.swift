// Copyright (c). Gem Wallet. All rights reserved.

import Observation
import Primitives

@Observable
@MainActor
public final class ConnectionStatusObserver {
    private let monitors: [any ConnectionComponentMonitoring]

    public private(set) var isHealthyByComponent: [ConnectionComponent: Bool] = [:]

    @ObservationIgnored private var tasks: [Task<Void, Never>] = []

    public nonisolated init(monitors: [any ConnectionComponentMonitoring] = [InternetConnectionMonitor()]) {
        self.monitors = monitors
    }

    public func start() {
        guard tasks.isEmpty else { return }
        tasks = monitors.map { monitor in
            Task { [weak self] in
                for await isHealthy in monitor.healthStream() {
                    self?.update(component: monitor.component, isHealthy: isHealthy)
                }
            }
        }
    }

    public func stop() {
        tasks.forEach { $0.cancel() }
        tasks = []
    }

    public var status: ConnectionStatus {
        isHealthyByComponent
            .filter { !$0.value }
            .keys
            .map(\.failureStatus)
            .max { $0.severity < $1.severity } ?? .online
    }

    func update(component: ConnectionComponent, isHealthy: Bool) {
        if component == .internet, isHealthy, isHealthyByComponent[.internet] == false {
            isHealthyByComponent = [:]
        }
        isHealthyByComponent[component] = isHealthy
    }
}
