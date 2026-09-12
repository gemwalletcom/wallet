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
            onEvent: { calls.continuation.yield($0) },
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
    func cancellationDuringPreparationDoesNotOpenSocket() async {
        let preparing = AsyncStream<CheckedContinuation<Void, Never>>.makeStream()
        let canceled = AsyncStream<Void>.makeStream()
        let opened = Locked(wrappedValue: false)
        let socket = WebSocketConnectionMock(onConnect: { opened.wrappedValue = true })
        let service = GemStreamServiceMock(prepare: {
            await withTaskCancellationHandler {
                await withCheckedContinuation { preparing.continuation.yield($0) }
            } onCancel: {
                canceled.continuation.yield(())
            }
            return true
        })
        let observer = StreamObserverService.mock(service: service, webSocket: socket)
        await observer.connect()
        var preparations = preparing.stream.makeAsyncIterator()
        let completion = await preparations.next()
        let stop = Task { await observer.disconnect() }
        var cancellations = canceled.stream.makeAsyncIterator()
        _ = await cancellations.next()
        completion?.resume()
        await stop.value
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
                preparations.wrappedValue += 1
                return true
            },
            onSession: { sessions.wrappedValue += 1 },
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
}
