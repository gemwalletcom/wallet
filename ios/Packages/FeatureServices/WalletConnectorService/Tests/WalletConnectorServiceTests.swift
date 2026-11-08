// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Testing
import WalletConnectorService
import WalletConnectorServiceTestKit
import WebSocketClientTestKit

struct WalletConnectorServiceTests {
    @Test
    func pairingsStartedTogetherOpenOneRelaySocket() async throws {
        let opened = AsyncStream<Void>.makeStream()
        let client = GemWalletConnectClientMock()
        let service = WalletConnectorService.mock(client: client, webSocket: WebSocketConnectionMock(onConnect: { opened.continuation.yield(()) }))

        async let first: Void = service.pair(uri: "wc:first")
        async let second: Void = service.pair(uri: "wc:second")
        _ = try await (first, second)

        #expect(await count(opened) == 1)
        #expect(await Set(client.pairedURIs) == ["wc:first", "wc:second"])
    }

    @Test
    func aDisconnectedRelayOpensAgainOnTheNextPairing() async throws {
        let opened = AsyncStream<Void>.makeStream()
        let service = WalletConnectorService.mock(webSocket: WebSocketConnectionMock(onConnect: { opened.continuation.yield(()) }))

        try await service.pair(uri: "wc:first")
        await service.disconnect()
        try await service.pair(uri: "wc:second")

        #expect(await count(opened) == 2)
    }

    private func count(_ stream: (stream: AsyncStream<Void>, continuation: AsyncStream<Void>.Continuation)) async -> Int {
        stream.continuation.finish()
        var count = 0
        for await _ in stream.stream {
            count += 1
        }
        return count
    }
}
