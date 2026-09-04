// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemConnectionService
import GemstonePrimitives
import Observation
import Primitives

@Observable
@MainActor
public final class ConnectionStatusObserver {
    private let connectionService: GemConnectionService
    private let monitors: [any ConnectionComponentMonitoring]

    public private(set) var isHealthyByComponent: [ConnectionComponent: Bool] = [:] {
        didSet {
            if status != status(for: oldValue) {
                debugLog("Connection status changed: \(status)")
            }
        }
    }

    @ObservationIgnored private var tasks: [Task<Void, Never>] = []

    public nonisolated init(
        connectionService: GemConnectionService,
        monitors: [any ConnectionComponentMonitoring],
    ) {
        self.connectionService = connectionService
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
        status(for: isHealthyByComponent)
    }

    func update(component: ConnectionComponent, isHealthy: Bool) {
        if connectionService.resetsComponentHealth(component: component.map(), isHealthy: isHealthy, wasHealthy: isHealthyByComponent[component]) {
            isHealthyByComponent = [:]
        }
        isHealthyByComponent[component] = isHealthy
    }

    private func status(for components: [ConnectionComponent: Bool]) -> ConnectionStatus {
        Array(components.filter { !$0.value }.keys).connectionStatus
    }
}
