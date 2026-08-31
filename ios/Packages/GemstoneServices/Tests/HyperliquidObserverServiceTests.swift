// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import GemstoneServices
import GemstonePrimitivesTestKit
import enum Gemstone.GemPerpetualSubscription
import protocol Gemstone.GemPerpetualStreamServiceProtocol
import typealias Gemstone.WalletId
import typealias Gemstone.PerpetualAccountMode
import typealias Gemstone.ChartCandleUpdate
import Primitives
import PrimitivesTestKit
import Testing
import WebSocketClient

private actor WebSocketConnectionStub: WebSocketConnectable {
    var state: WebSocketState = .disconnected

    func connect() -> AsyncStream<WebSocketEvent> {
        AsyncStream { continuation in
            continuation.yield(.connected)
            continuation.finish()
        }
    }

    func disconnect() async {}
    func send(_: Data) async throws {}
    func send(_: String) async throws {}
}

private final class PerpetualStreamServiceStub: GemPerpetualStreamServiceProtocol, @unchecked Sendable {
    private let lock = NSLock()
    private var connectedAddresses: [String] = []

    var addresses: [String] { lock.withLock { connectedAddresses } }

    func connected(address: String, mode _: PerpetualAccountMode) async throws {
        lock.withLock { connectedAddresses.append(address) }
    }

    func disconnected() async {}

    func handle(walletId _: WalletId, mode _: PerpetualAccountMode, data _: Data) async throws -> ChartCandleUpdate? {
        nil
    }

    func subscribe(subscription _: GemPerpetualSubscription) async throws {}
    func unsubscribe(subscription _: GemPerpetualSubscription) async throws {}
}

struct HyperliquidObserverServiceTests {
    @Test
    func retriesTheSameWalletAfterAFailedConnection() async throws {
        let wallet = Wallet.mock(accounts: [.mock(chain: .hyperCore)])
        let streamService = PerpetualStreamServiceStub()
        let perpetualService = GemPerpetualServiceMock()
        perpetualService.connectionFailures = 1
        let service = HyperliquidObserverService(
            webSocket: WebSocketConnectionStub(),
            perpetualService: perpetualService,
            streamService: streamService,
        )

        await service.setup(for: wallet)
        #expect(streamService.addresses.isEmpty)

        await service.setup(for: wallet)
        try await Task.sleep(for: .milliseconds(200))

        #expect(streamService.addresses.count == 1)
    }
}
