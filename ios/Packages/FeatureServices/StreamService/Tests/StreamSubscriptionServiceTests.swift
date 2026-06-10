// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import PrimitivesTestKit
import StreamServiceTestKit
import Testing
@testable import StreamService
import WebSocketClientTestKit

struct StreamSubscriptionServiceTests {
    private let decoder = JSONDecoder()

    @Test
    func setupBeforeConnectionSubscribesOnce() async throws {
        let webSocket = WebSocketConnectionMock()
        let service = StreamSubscriptionService.mock(webSocket: webSocket)

        try await service.setupAssets(walletId: .mock())
        #expect(await webSocket.getSentData().isEmpty)

        await webSocket.simulateConnected()
        await service.resubscribe()

        let messages = try await sentMessages(webSocket)
        #expect(messages.count == 1)
        #expect(Set(messages.first?.assets ?? []) == Set(AssetConfiguration.enabledByDefault))
    }

    @Test
    func setupSkipsSameAssets() async throws {
        let webSocket = WebSocketConnectionMock()
        let service = StreamSubscriptionService.mock(webSocket: webSocket)

        await webSocket.simulateConnected()
        try await service.setupAssets(walletId: .mock())
        try await service.setupAssets(walletId: .mock())
        await service.resubscribe()

        let messages = try await sentMessages(webSocket)
        #expect(messages.count == 1)
        #expect(Set(messages.first?.assets ?? []) == Set(AssetConfiguration.enabledByDefault))
    }

    @Test
    func resetAllowsReconnectResubscribe() async throws {
        let webSocket = WebSocketConnectionMock()
        let service = StreamSubscriptionService.mock(webSocket: webSocket)

        await webSocket.simulateConnected()
        try await service.setupAssets(walletId: .mock())
        await service.resetSubscriptions()
        await service.resubscribe()

        let messages = try await sentMessages(webSocket)
        #expect(messages.count == 2)
        #expect(Set(messages.last?.assets ?? []) == Set(AssetConfiguration.enabledByDefault))
    }

    private func sentMessages(_ webSocket: WebSocketConnectionMock) async throws -> [StreamMessagePrices] {
        try await webSocket.getSentData().compactMap { data in
            switch try decoder.decode(StreamMessage.self, from: data) {
            case let .subscribePrices(message):
                message
            default:
                nil
            }
        }
    }
}
