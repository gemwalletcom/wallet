// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import PerpetualService
import Primitives
import Testing
import WebSocketClientTestKit

struct HyperliquidSubscriptionServiceTests {
    @Test
    func sendsExactFramesThroughTheSocket() async throws {
        let webSocket = WebSocketConnectionMock()
        let service = HyperliquidSubscriptionService(webSocket: webSocket)

        try await service.subscribe(.marketPrices)
        #expect(await webSocket.getSentData().isEmpty)

        await webSocket.simulateConnected()
        try await service.connected(address: "0xabc", mode: .standard)

        let sent = await sentFrames(webSocket)
        #expect(sent.count == 3)
        #expect(Set(sent) == [
            #"{"method":"subscribe","subscription":{"type":"clearinghouseState","user":"0xabc"}}"#,
            #"{"method":"subscribe","subscription":{"type":"openOrders","user":"0xabc"}}"#,
            #"{"method":"subscribe","subscription":{"type":"allMids"}}"#,
        ])

        try await service.unsubscribe(.marketPrices)
        #expect(await sentFrames(webSocket).last == #"{"method":"unsubscribe","subscription":{"type":"allMids"}}"#)

        await service.disconnected()
        try await service.subscribe(.marketPrices)
        #expect(await webSocket.getSentData().count == 4)
    }

    private func sentFrames(_ webSocket: WebSocketConnectionMock) async -> [String] {
        await webSocket.getSentData().compactMap { String(data: $0, encoding: .utf8) }
    }
}
