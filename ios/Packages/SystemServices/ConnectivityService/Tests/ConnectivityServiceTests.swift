// Copyright (c). Gem Wallet. All rights reserved.

@testable import ConnectivityService
import ConnectivityServiceTestKit
import Testing

struct ConnectivityServiceTests {
    @Test
    func stateDerivation() {
        #expect(ConnectivityState.unknown.isOffline == false)
        #expect(ConnectivityState.satisfied.isOffline == false)
        #expect(ConnectivityState.unsatisfied(.noNetwork).isOffline == true)
    }

    @Test
    func observeDeliversCurrentThenUpdates() async throws {
        let (stream, continuation) = AsyncStream<ConnectivityState>.makeStream()
        let service = ConnectivityService.mock(monitor: ConnectivityMonitorMock(stream: stream))
        await service.start()

        var iterator = await service.observe().makeAsyncIterator()
        #expect(await iterator.next() == .unknown)

        continuation.yield(.satisfied)
        #expect(await iterator.next() == .satisfied)
    }
}
