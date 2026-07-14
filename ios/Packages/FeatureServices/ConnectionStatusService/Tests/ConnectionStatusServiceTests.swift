// Copyright (c). Gem Wallet. All rights reserved.

@testable import ConnectionStatusService
import ConnectivityService
import Primitives
import Testing

struct ConnectionStatusServiceTests {
    @Test
    func failureStatus() {
        #expect(ConnectionComponent.internet.failureStatus == .noInternet)
        #expect(ConnectionComponent.api.failureStatus == .noService)
        #expect(ConnectionComponent.nodes.failureStatus == .noService)
        #expect(ConnectionComponent.stream.failureStatus == .noService)
    }

    @Test
    func connectivityStateHealth() {
        let path = NetworkPath(transports: [.wifi], isExpensive: true, isConstrained: false, isVPN: true)

        #expect(ConnectivityState.unknown.health == ConnectionComponentHealth(isHealthy: true, metadata: .none))
        #expect(ConnectivityState.unsatisfied(.noNetwork).health == ConnectionComponentHealth(isHealthy: false, metadata: .none))
        #expect(ConnectivityState.satisfied(path).health == ConnectionComponentHealth(
            isHealthy: true,
            metadata: .internet(InternetConnectionMetadata(isExpensive: true, isConstrained: false, isVpn: true)),
        ))
    }

    @Test
    @MainActor
    func updateComponent() {
        let observer = ConnectionStatusObserver(monitors: [])

        #expect(observer.status == .online)

        observer.update(component: .api, health: ConnectionComponentHealth(isHealthy: false, metadata: .none))
        #expect(observer.status == .noService)

        observer.update(component: .internet, health: ConnectionComponentHealth(isHealthy: false, metadata: .none))
        #expect(observer.status == .noInternet)

        observer.update(component: .api, health: ConnectionComponentHealth(isHealthy: true, metadata: .none))
        observer.update(component: .internet, health: ConnectionComponentHealth(isHealthy: true, metadata: .none))
        #expect(observer.status == .online)
    }

    @Test
    @MainActor
    func internetRecoveryResetsComponents() {
        let observer = ConnectionStatusObserver(monitors: [])

        observer.update(component: .internet, health: ConnectionComponentHealth(isHealthy: false, metadata: .none))
        observer.update(component: .api, health: ConnectionComponentHealth(isHealthy: false, metadata: .none))
        observer.update(component: .nodes, health: ConnectionComponentHealth(isHealthy: false, metadata: .none))
        #expect(observer.status == .noInternet)

        observer.update(component: .internet, health: ConnectionComponentHealth(isHealthy: true, metadata: .none))
        #expect(observer.status == .online)
    }

    @Test
    @MainActor
    func internetHealthyDoesNotResetComponents() {
        let observer = ConnectionStatusObserver(monitors: [])

        observer.update(component: .internet, health: ConnectionComponentHealth(isHealthy: true, metadata: .none))
        observer.update(component: .api, health: ConnectionComponentHealth(isHealthy: false, metadata: .none))
        observer.update(component: .internet, health: ConnectionComponentHealth(isHealthy: true, metadata: .none))

        #expect(observer.status == .noService)
    }
}
