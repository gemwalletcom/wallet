// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import typealias Gemstone.ChartCandleUpdate
import protocol Gemstone.GemPerpetualStreamServiceProtocol
import enum Gemstone.GemPerpetualSubscription
import typealias Gemstone.PerpetualAccountMode
import typealias Gemstone.WalletId
import GemstonePrimitivesTestKit
@testable import GemstoneServices
import Primitives
import PrimitivesTestKit
import Testing
import WebSocketClientTestKit

private final class PerpetualStreamServiceStub: GemPerpetualStreamServiceProtocol, @unchecked Sendable {
    private let lock = NSLock()
    private var connectedAddresses: [String] = []

    var addresses: [String] { lock.withLock { connectedAddresses } }

    func connected(address: String, mode _: PerpetualAccountMode) async throws {
        lock.withLock { connectedAddresses.append(address) }
    }

    func disconnected() async {}

    func candleUpdate(walletId _: WalletId, mode _: PerpetualAccountMode, data _: Data) async throws -> ChartCandleUpdate? {
        nil
    }

    func subscribe(subscription _: GemPerpetualSubscription) async throws {}
    func unsubscribe(subscription _: GemPerpetualSubscription) async throws {}
}

struct HyperliquidObserverServiceTests {
    @Test(.timeLimit(.minutes(1)))
    func retriesTheSameWalletAfterAFailedConnection() async throws {
        let wallet = Wallet.mock(accounts: [.mock(chain: .hyperCore)])
        let opened = AsyncStream<Void>.makeStream()
        let socket = WebSocketConnectionMock(onConnect: { opened.continuation.yield(()) })
        let streamService = PerpetualStreamServiceStub()
        let perpetualService = GemPerpetualServiceMock()
        perpetualService.connectionFailures = 1
        let service = HyperliquidObserverService(
            webSocket: socket,
            perpetualService: perpetualService,
            streamService: streamService,
        )

        await service.setup(for: wallet)
        #expect(streamService.addresses.isEmpty)

        await service.setup(for: wallet)
        var connections = opened.stream.makeAsyncIterator()
        _ = await connections.next()
        await socket.simulateConnected()
        try await Task.sleep(for: .milliseconds(200))

        #expect(streamService.addresses.count == 1)
    }
}
