// Copyright (c). Gem Wallet. All rights reserved.

@testable import ConnectionStatusService
import ConnectionStatusServiceTestKit
import ConnectivityService
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
        let observer = ConnectionStatusObserver.mock()

        #expect(observer.status == .online)

        observer.update(component: .stream, isHealthy: false)
        #expect(observer.status == .noService)

        observer.update(component: .internet, isHealthy: false)
        #expect(observer.status == .noInternet)

        observer.update(component: .stream, isHealthy: true)
        observer.update(component: .internet, isHealthy: true)
        #expect(observer.status == .online)
    }

    @Test
    @MainActor
    func internetRecoveryResetsComponents() {
        let observer = ConnectionStatusObserver.mock()

        observer.update(component: .internet, isHealthy: false)
        observer.update(component: .stream, isHealthy: false)
        #expect(observer.status == .noInternet)

        observer.update(component: .internet, isHealthy: true)
        #expect(observer.status == .online)
        #expect(observer.isHealthyByComponent[.stream] == nil)
    }

    @Test
    @MainActor
    func internetHealthyDoesNotResetComponents() {
        let observer = ConnectionStatusObserver.mock()

        observer.update(component: .internet, isHealthy: true)
        observer.update(component: .stream, isHealthy: false)
        observer.update(component: .internet, isHealthy: true)

        #expect(observer.status == .noService)
    }
}
