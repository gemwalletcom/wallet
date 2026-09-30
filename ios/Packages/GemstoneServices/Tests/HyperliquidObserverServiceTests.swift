// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitivesTestKit
@testable import GemstoneServices
import Primitives
import PrimitivesTestKit
import Testing
import WebSocketClientTestKit

struct HyperliquidObserverServiceTests {
    @Test(.timeLimit(.minutes(1)))
    func retriesTheSameWalletAfterAFailedConnection() async throws {
        let wallet = Wallet.mock(accounts: [.mock(chain: .hyperCore)])
        let opened = AsyncStream<Void>.makeStream()
        let socket = WebSocketConnectionMock(onConnect: { opened.continuation.yield(()) })
        let streamService = GemPerpetualStreamServiceMock()
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
        let wallet = Wallet.mock(id: .mock(address: "0xa"), accounts: [.mock(chain: .hyperCore, address: "0xa")])
        let gate = AsyncStream<CheckedContinuation<Void, Never>>.makeStream()
        let opened = Locked(wrappedValue: 0)
        let socket = WebSocketConnectionMock(onConnect: { opened.withLock { $0 += 1 } })
        let perpetualService = GemPerpetualServiceMock()
        perpetualService.connectionGate = { _ in await withCheckedContinuation { gate.continuation.yield($0) } }
        let service = HyperliquidObserverService(webSocket: socket, perpetualService: perpetualService, streamService: GemPerpetualStreamServiceMock())

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
        let walletA = Wallet.mock(id: .mock(address: "0xa"), accounts: [.mock(chain: .hyperCore, address: "0xa")])
        let walletB = Wallet.mock(id: .mock(address: "0xb"), accounts: [.mock(chain: .hyperCore, address: "0xb")])
        let gate = AsyncStream<CheckedContinuation<Void, Never>>.makeStream()
        let opened = AsyncStream<Void>.makeStream()
        let socket = WebSocketConnectionMock(onConnect: { opened.continuation.yield(()) })
        let streamService = GemPerpetualStreamServiceMock()
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
        let wallet = Wallet.mock(id: .mock(address: "0xa"), accounts: [.mock(chain: .hyperCore, address: "0xa")])
        let gate = AsyncStream<CheckedContinuation<Void, Never>>.makeStream()
        let opened = Locked(wrappedValue: 0)
        let socket = WebSocketConnectionMock(onConnect: { opened.withLock { $0 += 1 } })
        let perpetualService = GemPerpetualServiceMock()
        perpetualService.connectionGate = { _ in await withCheckedContinuation { gate.continuation.yield($0) } }
        let service = HyperliquidObserverService(webSocket: socket, perpetualService: perpetualService, streamService: GemPerpetualStreamServiceMock())

        let setup = Task { await service.setup(for: wallet) }
        var pending = gate.stream.makeAsyncIterator()
        let preparation = await pending.next()
        await service.disconnect()
        preparation?.resume()
        await setup.value
        try? await Task.sleep(for: .milliseconds(100))

        #expect(opened.wrappedValue == 0)
    }
}
