// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import WalletConnectRelay

final class WebSocket: NSObject, @unchecked Sendable {
    @Locked var request: URLRequest
    @Locked private(set) var isConnected: Bool = false
    @Locked var onConnect: (() -> Void)?
    @Locked var onDisconnect: ((Error?) -> Void)?
    @Locked var onText: ((String) -> Void)?

    @Locked private var connection: Connection?

    private let delegateQueue: OperationQueue = {
        let queue = OperationQueue()
        queue.maxConcurrentOperationCount = 1
        return queue
    }()

    init(request: URLRequest) {
        _request = Locked(wrappedValue: request)
        super.init()
    }

    deinit {
        connection?.close(.goingAway)
    }

    var task: URLSessionWebSocketTask? {
        connection?.task
    }

    private func receiveMessage(on task: URLSessionWebSocketTask) {
        task.receive { [weak self] result in
            guard let self else { return }
            switch result {
            case let .success(message):
                switch message {
                case let .string(text):
                    onText?(text)
                case let .data(data):
                    if let text = try? data.encodeString() {
                        onText?(text)
                    }
                @unknown default:
                    break
                }
                receiveMessage(on: task)
            case .failure:
                break
            }
        }
    }

    private func completeAttempt(of task: URLSessionTask, error: Error?) {
        let isCurrentAttempt = _connection.withLock { connection in
            guard connection?.task === task, connection?.isComplete == false else { return false }
            connection?.isComplete = true
            return true
        }
        guard isCurrentAttempt else { return }
        isConnected = false
        onDisconnect?(error)
    }

    private func isCurrentTask(_ urlSessionTask: URLSessionTask) -> Bool {
        task === urlSessionTask
    }
}

private struct Connection {
    let session: URLSession
    let task: URLSessionWebSocketTask
    var isComplete = false

    func close(_ closeCode: URLSessionWebSocketTask.CloseCode) {
        task.cancel(with: closeCode, reason: nil)
        session.invalidateAndCancel()
    }
}

// MARK: - WebSocketConnecting

extension WebSocket: WebSocketConnecting {
    func connect() {
        let session = URLSession(configuration: .default, delegate: self, delegateQueue: delegateQueue)
        let task = session.webSocketTask(with: request)
        let replaced = _connection.withLock { connection in
            let replaced = connection
            connection = Connection(session: session, task: task)
            return replaced
        }
        replaced?.close(.goingAway)
        task.resume()
        receiveMessage(on: task)
    }

    func disconnect() {
        connection?.close(.normalClosure)
    }

    func write(string: String, completion: (() -> Void)?) {
        guard let task else {
            completion?()
            return
        }
        let sendableCompletion = UncheckedSendable(value: completion)
        task.send(.string(string)) { _ in
            sendableCompletion.value?()
        }
    }
}

// MARK: - URLSessionWebSocketDelegate

extension WebSocket: URLSessionWebSocketDelegate {
    func urlSession(_: URLSession, webSocketTask: URLSessionWebSocketTask, didOpenWithProtocol _: String?) {
        guard isCurrentTask(webSocketTask) else { return }
        isConnected = true
        onConnect?()
    }

    func urlSession(_: URLSession, webSocketTask: URLSessionWebSocketTask, didCloseWith _: URLSessionWebSocketTask.CloseCode, reason _: Data?) {
        completeAttempt(of: webSocketTask, error: nil)
    }

    func urlSession(_: URLSession, task: URLSessionTask, didCompleteWithError error: Error?) {
        completeAttempt(of: task, error: error)
    }
}
