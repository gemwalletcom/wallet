// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitivesTestKit
import Primitives
import StreamService
import StreamServiceTestKit
import Testing
import WebSocketClientTestKit

@Suite(.timeLimit(.minutes(1)))
struct StreamObserverServiceTests {
    @Test
    func forwardsEventsInOrder() async {
        let opened = AsyncStream<Void>.makeStream()
        let calls = AsyncStream<String>.makeStream()
        let connected = AsyncStream<CheckedContinuation<Void, Never>>.makeStream()
        let socket = WebSocketConnectionMock(onConnect: { opened.continuation.yield(()) })
        let service = GemStreamServiceMock(
            onConnected: {
                await withCheckedContinuation { connected.continuation.yield($0) }
                calls.continuation.yield("connected")
            },
            onDisconnected: { calls.continuation.yield("disconnected") },
            onEvent: {
                calls.continuation.yield($0)
                return .prices(prices: 1, rates: 0)
            },
        )
        let observer = StreamObserverService.mock(service: service, webSocket: socket)
        await observer.connect()
        var connections = opened.stream.makeAsyncIterator()
        _ = await connections.next()
        await socket.simulateConnected()
        var pending = connected.stream.makeAsyncIterator()
        let completion = await pending.next()
        await socket.simulateMessage(Data("message".utf8))
        await socket.simulateDisconnect()
        completion?.resume()
        var events = calls.stream.makeAsyncIterator()
        #expect(await events.next() == "connected")
        #expect(await events.next() == "message")
        #expect(await events.next() == "disconnected")
        await observer.disconnect()
    }

    @Test
    func slowSyncDoesNotHoldBackTheNextEvent() async {
        let opened = AsyncStream<Void>.makeStream()
        let handled = AsyncStream<String>.makeStream()
        let syncing = AsyncStream<CheckedContinuation<Void, Never>>.makeStream()
        let socket = WebSocketConnectionMock(onConnect: { opened.continuation.yield(()) })
        let service = GemStreamServiceMock(
            onEvent: {
                handled.continuation.yield($0)
                return $0 == "balances" ? .balances(walletId: "multicoin_0x1", assetIds: []) : .prices(prices: 1, rates: 0)
            },
            onSync: { event in
                guard case .balances = event else { return }
                await withCheckedContinuation { syncing.continuation.yield($0) }
            },
        )
        let observer = StreamObserverService.mock(service: service, webSocket: socket)
        await observer.connect()
        var connections = opened.stream.makeAsyncIterator()
        _ = await connections.next()
        await socket.simulateConnected()
        await socket.simulateMessage(Data("balances".utf8))
        var syncs = syncing.stream.makeAsyncIterator()
        let pendingSync = await syncs.next()
        await socket.simulateMessage(Data("prices".utf8))
        var events = handled.stream.makeAsyncIterator()
        #expect(await events.next() == "balances")
        #expect(await events.next() == "prices")
        pendingSync?.resume()
        await observer.disconnect()
    }

    @Test
    func cancellationDuringPreparationDoesNotOpenSocket() async {
        let preparing = AsyncStream<CheckedContinuation<Void, Never>>.makeStream()
        let canceled = AsyncStream<Void>.makeStream()
        let closed = AsyncStream<Void>.makeStream()
        let opened = Locked(wrappedValue: false)
        let socket = WebSocketConnectionMock(onConnect: { opened.wrappedValue = true })
        let service = GemStreamServiceMock(
            prepare: {
                await withTaskCancellationHandler {
                    await withCheckedContinuation { preparing.continuation.yield($0) }
                } onCancel: {
                    canceled.continuation.yield(())
                }
                return true
            },
            onDisconnected: { closed.continuation.yield(()) },
        )
        let observer = StreamObserverService.mock(service: service, webSocket: socket)
        await observer.connect()
        var preparations = preparing.stream.makeAsyncIterator()
        let completion = await preparations.next()
        await observer.disconnect()
        var cancellations = canceled.stream.makeAsyncIterator()
        _ = await cancellations.next()
        completion?.resume()
        var closures = closed.stream.makeAsyncIterator()
        _ = await closures.next()
        #expect(opened.wrappedValue == false)
    }

    @Test
    func sessionUpdateKeepsTheConnectionAndStopsWithTheObserver() async {
        let opened = AsyncStream<Void>.makeStream()
        let preparations = Locked(wrappedValue: 0)
        let sessions = Locked(wrappedValue: 0)
        let socket = WebSocketConnectionMock(onConnect: { opened.continuation.yield(()) })
        let service = GemStreamServiceMock(
            prepare: {
                preparations.withLock { $0 += 1 }
                return true
            },
            onSession: { sessions.withLock { $0 += 1 } },
        )
        let observer = StreamObserverService.mock(service: service, webSocket: socket)
        var connections = opened.stream.makeAsyncIterator()
        await observer.connect()
        _ = await connections.next()

        await observer.updateSession()

        #expect(sessions.wrappedValue == 1)
        #expect(preparations.wrappedValue == 1)

        await observer.disconnect()
        await observer.updateSession()

        #expect(preparations.wrappedValue == 1)
    }

    @Test
    func socketReportsHealthWhileTheObserverRuns() async {
        let opened = AsyncStream<Void>.makeStream()
        let socket = WebSocketConnectionMock(onConnect: { opened.continuation.yield(()) })
        let health = ConnectionComponentHealth(component: .stream)
        let observer = StreamObserverService.mock(webSocket: socket, health: health)
        var reports = health.healthStream().makeAsyncIterator()
        var connections = opened.stream.makeAsyncIterator()
        await observer.connect()
        _ = await connections.next()

        await socket.simulateConnected()
        #expect(await reports.next() == true)

        await socket.simulateDisconnect()
        #expect(await reports.next() == false)

        await observer.disconnect()
    }

    @Test
    func foregroundSessionUpdateConnectsWhenCoreBecomesReady() async {
        let opened = AsyncStream<Void>.makeStream()
        let closed = AsyncStream<Void>.makeStream()
        let ready = Locked(wrappedValue: false)
        let service = GemStreamServiceMock(
            prepare: { ready.wrappedValue },
            onDisconnected: { closed.continuation.yield(()) },
        )
        let socket = WebSocketConnectionMock(onConnect: { opened.continuation.yield(()) })
        let observer = StreamObserverService.mock(service: service, webSocket: socket)
        await observer.connect()
        var closures = closed.stream.makeAsyncIterator()
        _ = await closures.next()
        ready.wrappedValue = true
        await observer.updateSession()
        var connections = opened.stream.makeAsyncIterator()
        _ = await connections.next()
        await observer.disconnect()
    }

    @Test(arguments: [false, true])
    func foregroundReconnectsAcrossCleanup(resumeBeforeCleanup: Bool) async {
        let opened = AsyncStream<Bool>.makeStream()
        let cleanup = AsyncStream<CheckedContinuation<Void, Never>>.makeStream()
        let closed = AsyncStream<Int>.makeStream()
        let disconnects = Locked(wrappedValue: 0)
        let socket = WebSocketConnectionMock(onConnect: { opened.continuation.yield(true) })
        let service = GemStreamServiceMock(onDisconnected: {
            disconnects.withLock { $0 += 1 }
            if disconnects.wrappedValue == 1 {
                await withCheckedContinuation { cleanup.continuation.yield($0) }
            }
            closed.continuation.yield(disconnects.wrappedValue)
        })
        let observer = StreamObserverService.mock(service: service, webSocket: socket)
        await observer.connect()
        var connections = opened.stream.makeAsyncIterator()
        #expect(await connections.next() == true)

        await observer.disconnect()
        var cleanups = cleanup.stream.makeAsyncIterator()
        let release = await cleanups.next()
        if resumeBeforeCleanup {
            await observer.connect()
        }
        release?.resume()
        var closures = closed.stream.makeAsyncIterator()
        #expect(await closures.next() == 1)
        if !resumeBeforeCleanup {
            await observer.connect()
        }

        let reconnected = await withTaskGroup(of: Bool.self) { group in
            group.addTask {
                var events = opened.stream.makeAsyncIterator()
                return await events.next() == true
            }
            group.addTask {
                try? await Task.sleep(for: .seconds(5))
                return false
            }
            let result = await group.next() ?? false
            group.cancelAll()
            return result
        }
        #expect(reconnected, "Foreground must open a replacement connection, including during cleanup")
        await observer.disconnect()
        #expect(await closures.next() == 2)
    }

    @Test
    func backgroundDuringCleanupStaysDisconnected() async {
        let opened = AsyncStream<Bool>.makeStream()
        let cleanup = AsyncStream<CheckedContinuation<Void, Never>>.makeStream()
        let closed = AsyncStream<Void>.makeStream()
        let disconnects = Locked(wrappedValue: 0)
        let socket = WebSocketConnectionMock(onConnect: { opened.continuation.yield(true) })
        let service = GemStreamServiceMock(onDisconnected: {
            disconnects.withLock { $0 += 1 }
            if disconnects.wrappedValue == 1 {
                await withCheckedContinuation { cleanup.continuation.yield($0) }
            }
            closed.continuation.yield(())
        })
        let observer = StreamObserverService.mock(service: service, webSocket: socket)
        await observer.connect()
        var connections = opened.stream.makeAsyncIterator()
        #expect(await connections.next() == true)

        await observer.disconnect()
        var cleanups = cleanup.stream.makeAsyncIterator()
        let release = await cleanups.next()
        await observer.connect()
        await observer.disconnect()
        release?.resume()
        var closures = closed.stream.makeAsyncIterator()
        _ = await closures.next()

        let reconnected = await withTaskGroup(of: Bool.self) { group in
            group.addTask {
                var events = opened.stream.makeAsyncIterator()
                return await events.next() == true
            }
            group.addTask {
                try? await Task.sleep(for: .milliseconds(200))
                return false
            }
            let result = await group.next() ?? false
            group.cancelAll()
            return result
        }
        #expect(!reconnected)
        #expect(await socket.state == .disconnected)
        await observer.disconnect()
    }

    @Test
    func repeatedForegroundDuringCleanupOpensOneReplacement() async {
        let opened = AsyncStream<Int>.makeStream()
        let cleanup = AsyncStream<CheckedContinuation<Void, Never>>.makeStream()
        let closed = AsyncStream<Int>.makeStream()
        let connectionCount = Locked(wrappedValue: 0)
        let disconnects = Locked(wrappedValue: 0)
        let socket = WebSocketConnectionMock(onConnect: {
            connectionCount.withLock { $0 += 1 }
            opened.continuation.yield(connectionCount.wrappedValue)
        })
        let service = GemStreamServiceMock(onDisconnected: {
            disconnects.withLock { $0 += 1 }
            if disconnects.wrappedValue == 1 {
                await withCheckedContinuation { cleanup.continuation.yield($0) }
            }
            closed.continuation.yield(disconnects.wrappedValue)
        })
        let observer = StreamObserverService.mock(service: service, webSocket: socket)
        await observer.connect()
        var connections = opened.stream.makeAsyncIterator()
        #expect(await connections.next() == 1)

        await observer.disconnect()
        var cleanups = cleanup.stream.makeAsyncIterator()
        let release = await cleanups.next()
        for _ in 0 ..< 3 {
            await observer.connect()
        }
        #expect(connectionCount.wrappedValue == 1)
        release?.resume()
        #expect(await connections.next() == 2)
        for _ in 0 ..< 3 {
            await observer.connect()
        }

        await observer.disconnect()
        var closures = closed.stream.makeAsyncIterator()
        #expect(await closures.next() == 1)
        #expect(await closures.next() == 2)
        #expect(connectionCount.wrappedValue == 2)
    }
}
