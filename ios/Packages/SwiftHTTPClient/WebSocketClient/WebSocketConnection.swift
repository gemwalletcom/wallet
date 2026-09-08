// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public actor WebSocketConnection: WebSocketConnectable {
    public private(set) var state: WebSocketState = .disconnected

    private let configuration: WebSocketConfiguration

    private var session: URLSession?
    private var task: URLSessionWebSocketTask?
    private var reconnectTask: Task<Void, Never>?
    private var keepaliveTask: Task<Void, Never>?
    private var continuation: AsyncStream<WebSocketEvent>.Continuation?
    private var streamId: UUID?
    private var connectionId: UUID?
    private var reconnectAttempt: Int = 0
    private var pendingMessages: [URLSessionWebSocketTask.Message] = []

    public init(configuration: WebSocketConfiguration) {
        self.configuration = configuration
    }

    public init(url: URL, reconnection: any Reconnectable) {
        self.init(configuration: WebSocketConfiguration(url: url, reconnection: reconnection))
    }

    deinit {
        task?.cancel(with: .goingAway, reason: nil)
        reconnectTask?.cancel()
        keepaliveTask?.cancel()
        continuation?.finish()
    }

    // MARK: - Public

    public func connect() -> AsyncStream<WebSocketEvent> {
        let (stream, continuation) = AsyncStream<WebSocketEvent>.makeStream()
        setupStream(continuation)
        return stream
    }

    public func disconnect() async {
        state = .disconnected
        streamId = nil

        cancelReconnect()
        cancelTask()
        cancelPendingMessages()
        invalidateSession()

        continuation?.yield(.disconnected(nil))
        continuation?.finish()
        continuation = nil
    }

    public func send(_ data: Data) async throws {
        try await send(message: .data(data))
    }

    public func send(_ text: String) async throws {
        try await send(message: .string(text))
    }

    // MARK: - Private

    private func send(message: URLSessionWebSocketTask.Message) async throws {
        switch state {
        case .connected:
            guard let task else { throw WebSocketError.notConnected }
            try await task.send(message)
        case .connecting, .reconnecting:
            pendingMessages.append(message)
        case .disconnected:
            throw WebSocketError.notConnected
        }
    }

    private func sendPendingMessages(connectionId: UUID) async {
        guard self.connectionId == connectionId, let task else { return }
        let messages = pendingMessages
        pendingMessages = []

        for message in messages {
            guard self.connectionId == connectionId, !Task.isCancelled else { return }
            do {
                try await task.send(message)
            } catch {
                #if DEBUG
                    NSLog("WebSocket send error: \(error)")
                #endif
            }
        }
    }

    private func cancelPendingMessages() {
        pendingMessages.removeAll()
    }

    private func cancelTask() {
        cancelKeepalive()
        connectionId = nil
        task?.cancel(with: .goingAway, reason: nil)
        task = nil
    }

    private func cancelKeepalive() {
        keepaliveTask?.cancel()
        keepaliveTask = nil
    }

    private func startKeepalive() {
        cancelKeepalive()
        let interval = configuration.reconnection.pingIntervalMilliseconds()
        keepaliveTask = Task { [weak self] in
            while !Task.isCancelled {
                try? await Task.sleep(for: .milliseconds(interval))
                guard !Task.isCancelled else { return }
                await self?.sendPing()
            }
        }
    }

    private func sendPing() {
        guard state == .connected, let task, let connectionId else { return }
        task.sendPing { [weak self] error in
            guard let error else { return }
            Task { await self?.handleError(error, connectionId: connectionId) }
        }
    }

    private func cancelReconnect() {
        reconnectTask?.cancel()
        reconnectTask = nil
    }

    private func invalidateSession() {
        session?.invalidateAndCancel()
        session = nil
    }

    private func setupStream(_ continuation: AsyncStream<WebSocketEvent>.Continuation) {
        self.continuation = continuation
        let streamId = UUID()
        self.streamId = streamId

        continuation.onTermination = { [weak self] _ in
            Task {
                await self?.handleStreamTermination(streamId: streamId)
            }
        }

        resetReconnectionAttempt()
        startConnection()
    }

    private func handleStreamTermination(streamId: UUID) {
        guard self.streamId == streamId, state != .disconnected else { return }
        self.streamId = nil

        cancelTask()
        cancelReconnect()
        cancelPendingMessages()
        invalidateSession()

        state = .disconnected
    }

    private func startConnection() {
        state = .connecting

        guard let request = try? configuration.requestProvider.makeRequest() else {
            scheduleReconnect(with: WebSocketError.notConnected)
            return
        }
        let connectionId = UUID()
        self.connectionId = connectionId

        let delegate = WebSocketSessionDelegate(
            didOpen: { [weak self] in
                Task { await self?.didOpen(connectionId: connectionId) }
            },
            didClose: { [weak self] closeCode, reason in
                Task { await self?.didClose(connectionId: connectionId, closeCode: closeCode, reason: reason) }
            },
        )

        session = URLSession(
            configuration: configuration.sessionConfiguration,
            delegate: delegate,
            delegateQueue: nil,
        )

        task = session?.webSocketTask(with: request)
        task?.resume()

        listen(connectionId: connectionId)
    }

    private func didOpen(connectionId: UUID) {
        guard self.connectionId == connectionId, state == .connecting else { return }

        state = .connected
        resetReconnectionAttempt()
        startKeepalive()
        continuation?.yield(.connected)

        Task {
            await sendPendingMessages(connectionId: connectionId)
        }
    }

    private func didClose(connectionId: UUID, closeCode _: URLSessionWebSocketTask.CloseCode, reason _: Data?) {
        guard self.connectionId == connectionId, state != .disconnected else { return }
        scheduleReconnect(with: nil)
    }

    private func listen(connectionId: UUID) {
        task?.receive { [weak self] result in
            Task {
                await self?.handleReceive(result, connectionId: connectionId)
            }
        }
    }

    private func handleReceive(_ result: Result<URLSessionWebSocketTask.Message, Error>, connectionId: UUID) {
        guard self.connectionId == connectionId else { return }
        switch result {
        case let .success(message):
            handleMessage(message)
            listen(connectionId: connectionId)

        case let .failure(error):
            handleError(error, connectionId: connectionId)
        }
    }

    private func handleMessage(_ message: URLSessionWebSocketTask.Message) {
        switch message {
        case let .data(data):
            continuation?.yield(.message(data))
        case let .string(text):
            if let data = text.data(using: .utf8) {
                continuation?.yield(.message(data))
            }
        @unknown default:
            break
        }
    }

    private func handleError(_ error: Error, connectionId: UUID) {
        guard self.connectionId == connectionId, state != .disconnected else { return }

        if let urlError = error as? URLError, urlError.code == .cancelled {
            return
        }

        scheduleReconnect(with: error)
    }

    private func scheduleReconnect(with error: Error?) {
        guard reconnectTask == nil else { return }

        cancelTask()

        guard state != .disconnected else { return }

        state = .reconnecting
        continuation?.yield(.disconnected(error))

        let delay = configuration.reconnection.reconnectDelayMilliseconds(attempt: UInt32(clamping: reconnectAttempt))
        reconnectAttempt += 1

        reconnectTask = Task { [weak self] in
            do {
                try await Task.sleep(for: .milliseconds(delay))
            } catch {
                return
            }
            await self?.finishReconnect()
        }
    }

    private func finishReconnect() {
        guard !Task.isCancelled else { return }
        reconnectTask = nil

        guard state == .reconnecting, task == nil else { return }
        startConnection()
    }

    private func resetReconnectionAttempt() {
        reconnectAttempt = 0
    }
}
