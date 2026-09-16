// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemStreamConnection
import WebSocketClient

public final class GemstoneStreamConnection: GemStreamConnection, Sendable {
    private let webSocket: any WebSocketConnectable

    public init(webSocket: any WebSocketConnectable) {
        self.webSocket = webSocket
    }

    public func isConnected() async -> Bool {
        await webSocket.state == .connected
    }

    public func send(message: String) async throws {
        try await webSocket.send(message)
    }
}
