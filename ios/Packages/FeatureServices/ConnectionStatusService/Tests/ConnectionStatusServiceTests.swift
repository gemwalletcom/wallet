// Copyright (c). Gem Wallet. All rights reserved.

@testable import ConnectionStatusService
import ConnectivityService
import class Gemstone.GemConnectionService
import Primitives
import Testing

struct ConnectionStatusServiceTests {
    @Test
    func connectivityStateIsOffline() {
        #expect(ConnectivityState.unknown.isOffline == false)
        #expect(ConnectivityState.satisfied.isOffline == false)
        #expect(ConnectivityState.unsatisfied(.noNetwork).isOffline == true)
    }

    @Test
    @MainActor
    func updateComponent() {
        let observer = ConnectionStatusObserver(connectionService: GemConnectionService(), monitors: [])

        #expect(observer.status == .online)

        observer.update(component: .api, isHealthy: false)
        #expect(observer.status == .noService)

        observer.update(component: .internet, isHealthy: false)
        #expect(observer.status == .noInternet)

        observer.update(component: .api, isHealthy: true)
        observer.update(component: .internet, isHealthy: true)
        #expect(observer.status == .online)
    }

    @Test
    @MainActor
    func internetRecoveryResetsComponents() {
        let observer = ConnectionStatusObserver(connectionService: GemConnectionService(), monitors: [])

        observer.update(component: .internet, isHealthy: false)
        observer.update(component: .api, isHealthy: false)
        observer.update(component: .nodes, isHealthy: false)
        #expect(observer.status == .noInternet)

        observer.update(component: .internet, isHealthy: true)
        #expect(observer.status == .online)
        #expect(observer.isHealthyByComponent[.api] == nil)
    }

    @Test
    @MainActor
    func internetHealthyDoesNotResetComponents() {
        let observer = ConnectionStatusObserver(connectionService: GemConnectionService(), monitors: [])

        observer.update(component: .internet, isHealthy: true)
        observer.update(component: .api, isHealthy: false)
        observer.update(component: .internet, isHealthy: true)

        #expect(observer.status == .noService)
    }
}
