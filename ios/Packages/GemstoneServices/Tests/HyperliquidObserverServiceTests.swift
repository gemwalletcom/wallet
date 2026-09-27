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

    @Test(.timeLimit(.minutes(1)))
    func overlappingSetupForTheSameWalletPreparesOnce() async {
        let wallet = hyperliquidWallet("0xa")
        let gate = AsyncStream<CheckedContinuation<Void, Never>>.makeStream()
        let opened = Locked(wrappedValue: 0)
        let socket = WebSocketConnectionMock(onConnect: { opened.withLock { $0 += 1 } })
        let perpetualService = GemPerpetualServiceMock()
        perpetualService.connectionGate = { _ in await withCheckedContinuation { gate.continuation.yield($0) } }
        let service = HyperliquidObserverService(webSocket: socket, perpetualService: perpetualService, streamService: PerpetualStreamServiceStub())

        let first = Task { await service.setup(for: wallet) }
        var pending = gate.stream.makeAsyncIterator()
        let preparation = await pending.next()
        await service.setup(for: wallet)
        preparation?.resume()
        await first.value
        try? await Task.sleep(for: .milliseconds(100))

        #expect(perpetualService.connectionCount == 1)
        #expect(opened.wrappedValue == 1)
    }

    @Test(.timeLimit(.minutes(1)))
    func aDelayedPreparationForTheOldWalletDoesNotReplaceTheNewOne() async {
        let walletA = hyperliquidWallet("0xa")
        let walletB = hyperliquidWallet("0xb")
        let gate = AsyncStream<CheckedContinuation<Void, Never>>.makeStream()
        let opened = AsyncStream<Void>.makeStream()
        let socket = WebSocketConnectionMock(onConnect: { opened.continuation.yield(()) })
        let streamService = PerpetualStreamServiceStub()
        let perpetualService = GemPerpetualServiceMock()
        perpetualService.connectionGate = { wallet in
            guard wallet.id == walletA.id else { return }
            await withCheckedContinuation { gate.continuation.yield($0) }
        }
        let service = HyperliquidObserverService(webSocket: socket, perpetualService: perpetualService, streamService: streamService)

        let setupA = Task { await service.setup(for: walletA) }
        var pending = gate.stream.makeAsyncIterator()
        let preparationA = await pending.next()
        await service.setup(for: walletB)
        var connections = opened.stream.makeAsyncIterator()
        _ = await connections.next()
        await socket.simulateConnected()
        preparationA?.resume()
        await setupA.value
        try? await Task.sleep(for: .milliseconds(100))

        #expect(streamService.addresses == ["0xb"])
    }

    @Test(.timeLimit(.minutes(1)))
    func goingToTheBackgroundDuringPreparationDoesNotConnect() async {
        let wallet = hyperliquidWallet("0xa")
        let gate = AsyncStream<CheckedContinuation<Void, Never>>.makeStream()
        let opened = Locked(wrappedValue: 0)
        let socket = WebSocketConnectionMock(onConnect: { opened.withLock { $0 += 1 } })
        let perpetualService = GemPerpetualServiceMock()
        perpetualService.connectionGate = { _ in await withCheckedContinuation { gate.continuation.yield($0) } }
        let service = HyperliquidObserverService(webSocket: socket, perpetualService: perpetualService, streamService: PerpetualStreamServiceStub())

        let setup = Task { await service.setup(for: wallet) }
        var pending = gate.stream.makeAsyncIterator()
        let preparation = await pending.next()
        await service.disconnect()
        preparation?.resume()
        await setup.value
        try? await Task.sleep(for: .milliseconds(100))

        #expect(opened.wrappedValue == 0)
    }

    private func hyperliquidWallet(_ address: String) -> Wallet {
        Wallet.mock(id: .mock(address: address), accounts: [.mock(chain: .hyperCore, address: address)])
    }
}
