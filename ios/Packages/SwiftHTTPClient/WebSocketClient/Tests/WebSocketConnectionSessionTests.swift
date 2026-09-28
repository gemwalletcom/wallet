// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Testing
@testable import WebSocketClient
import WebSocketClientTestKit

@Suite(.timeLimit(.minutes(1)))
struct WebSocketConnectionSessionTests {
    @Test
    func repeatedFailedConnectionsAndDisconnectRetireEverySessionAndDelegate() async throws {
        let recorder = SessionRecorder()
        let connection = WebSocketConnection(configuration: recorder.configuration())

        let events = await connection.connect()
        var iterator = events.makeAsyncIterator()
        while recorder.created < 4 {
            _ = await iterator.next()
        }
        await connection.disconnect()
        try await recorder.waitUntilRetired(recorder.created)

        #expect(recorder.retired == recorder.created)
        #expect(recorder.liveDelegates == 0)
    }

    @Test
    func aCallbackFromARetiredSessionIsIgnored() async throws {
        let recorder = SessionRecorder()
        let connection = WebSocketConnection(configuration: recorder.configuration())

        let events = await connection.connect()
        var iterator = events.makeAsyncIterator()
        while recorder.created < 2 {
            _ = await iterator.next()
        }
        let retired = try #require(recorder.firstDelegate)
        let url = try #require(URL(string: "ws://127.0.0.1:9"))
        retired.urlSession?(URLSession.shared, webSocketTask: URLSession.shared.webSocketTask(with: url), didOpenWithProtocol: nil)
        try await Task.sleep(for: .milliseconds(50))

        #expect(await connection.state != .connected)
        await connection.disconnect()
    }
}

private final class SessionRecorder: @unchecked Sendable {
    private let lock = NSLock()
    private var createdCount = 0
    private var retiredCount = 0
    private var delegates: [WeakDelegate] = []
    private var first: (any URLSessionWebSocketDelegate)?

    var created: Int { lock.withLock { createdCount } }
    var retired: Int { lock.withLock { retiredCount } }
    var liveDelegates: Int { lock.withLock { delegates.count(where: { $0.value != nil }) } }
    var firstDelegate: (any URLSessionWebSocketDelegate)? { lock.withLock { first } }

    func configuration() -> WebSocketConfiguration {
        WebSocketConfiguration(
            requestProvider: StaticRequestProvider(url: URL(string: "ws://127.0.0.1:9")!),
            reconnection: ReconnectableMock(),
            makeSession: { [self] configuration, delegate in
                let observer = InvalidationObserver(inner: delegate) { [self] in
                    lock.withLock { retiredCount += 1 }
                }
                lock.withLock {
                    createdCount += 1
                    delegates.append(WeakDelegate(value: observer))
                    if first == nil, let delegate = delegate as? any URLSessionWebSocketDelegate {
                        first = delegate
                    }
                }
                return URLSession(configuration: configuration, delegate: observer, delegateQueue: nil)
            },
        )
    }

    func waitUntilRetired(_ count: Int) async throws {
        while retired < count {
            try await Task.sleep(for: .milliseconds(20))
        }
        try await Task.sleep(for: .milliseconds(50))
    }
}

private final class WeakDelegate: @unchecked Sendable {
    weak var value: AnyObject?

    init(value: AnyObject) {
        self.value = value
    }
}

private final class InvalidationObserver: NSObject, URLSessionWebSocketDelegate, @unchecked Sendable {
    private let inner: any URLSessionDelegate
    private let onInvalid: () -> Void

    init(inner: any URLSessionDelegate, onInvalid: @escaping () -> Void) {
        self.inner = inner
        self.onInvalid = onInvalid
    }

    func urlSession(_: URLSession, didBecomeInvalidWithError _: Error?) {
        onInvalid()
    }

    func urlSession(_ session: URLSession, webSocketTask: URLSessionWebSocketTask, didOpenWithProtocol protocolName: String?) {
        (inner as? any URLSessionWebSocketDelegate)?.urlSession?(session, webSocketTask: webSocketTask, didOpenWithProtocol: protocolName)
    }

    func urlSession(_ session: URLSession, webSocketTask: URLSessionWebSocketTask, didCloseWith closeCode: URLSessionWebSocketTask.CloseCode, reason: Data?) {
        (inner as? any URLSessionWebSocketDelegate)?.urlSession?(session, webSocketTask: webSocketTask, didCloseWith: closeCode, reason: reason)
    }
}
