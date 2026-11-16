// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemSocketTransport
import WebSocketClient

public final class GemstoneSocketTransport: GemSocketTransport, Sendable {
    private let webSocket: any WebSocketConnectable

    public init(webSocket: any WebSocketConnectable) {
        self.webSocket = webSocket
    }

    public func send(message: String) async throws {
        try await webSocket.send(message)
    }
}
