// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemPerpetualStreamConnection
import WebSocketClient

public final class PerpetualStreamConnection: GemPerpetualStreamConnection, Sendable {
    private let webSocket: any WebSocketConnectable

    public init(webSocket: any WebSocketConnectable) {
        self.webSocket = webSocket
    }

    public func send(message: String) async throws {
        try await webSocket.send(message)
    }
}
