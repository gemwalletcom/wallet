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
        let closed = AsyncStream<Void>.makeStream()
        let socket = WebSocketConnectionMock(onConnect: { opened.continuation.yield(()) }, onDisconnect: { closed.continuation.yield(()) })
        let streamService = GemPerpetualStreamServiceMock()
        let perpetualService = GemPerpetualServiceMock()
        perpetualService.connectionFailures = 1
        let service = HyperliquidObserverService(
            webSocket: socket,
            perpetualService: perpetualService,
            streamService: streamService,
        )

        await service.setup(for: wallet)
        _ = await closed.stream.first { _ in true }
        #expect(streamService.addresses.isEmpty)

        await service.setup(for: wallet)
        for await _ in opened.stream.prefix(2) {}
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

        await service.setup(for: walletA)
        let preparationA = await gate.stream.first { _ in true }
        await service.setup(for: walletB)
        for await _ in opened.stream.prefix(2) {}
        await socket.simulateConnected()
        preparationA?.resume()
        try? await Task.sleep(for: .milliseconds(100))

        #expect(streamService.addresses == ["0xb"])
    }

    @Test(.timeLimit(.minutes(1)))
    func goingToTheBackgroundDuringPreparationClosesTheSocket() async {
        let wallet = Wallet.mock(id: .mock(address: "0xa"), accounts: [.mock(chain: .hyperCore, address: "0xa")])
        let gate = AsyncStream<CheckedContinuation<Void, Never>>.makeStream()
        let socket = WebSocketConnectionMock()
        let perpetualService = GemPerpetualServiceMock()
        perpetualService.connectionGate = { _ in await withCheckedContinuation { gate.continuation.yield($0) } }
        let service = HyperliquidObserverService(webSocket: socket, perpetualService: perpetualService, streamService: GemPerpetualStreamServiceMock())

        await service.setup(for: wallet)
        let preparation = await gate.stream.first { _ in true }
        await service.disconnect()
        preparation?.resume()
        try? await Task.sleep(for: .milliseconds(100))

        #expect(await socket.state == .disconnected)
    }

    @Test(.timeLimit(.minutes(1)))
    func opensTheSocketWhileTheAccountModeLoads() async throws {
        let wallet = Wallet.mock(id: .mock(address: "0xa"), accounts: [.mock(chain: .hyperCore, address: "0xa")])
        let gate = AsyncStream<CheckedContinuation<Void, Never>>.makeStream()
        let socket = WebSocketConnectionMock()
        let streamService = GemPerpetualStreamServiceMock()
        let perpetualService = GemPerpetualServiceMock()
        perpetualService.connectionGate = { _ in await withCheckedContinuation { gate.continuation.yield($0) } }
        let service = HyperliquidObserverService(webSocket: socket, perpetualService: perpetualService, streamService: streamService)

        await service.setup(for: wallet)
        let preparation = await gate.stream.first { _ in true }
        #expect(await socket.state == .connecting)

        await socket.simulateConnected()
        try await Task.sleep(for: .milliseconds(100))
        #expect(streamService.addresses.isEmpty)

        preparation?.resume()
        try await Task.sleep(for: .milliseconds(100))
        #expect(streamService.addresses == ["0xa"])
    }
}
