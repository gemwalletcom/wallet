// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import func Gemstone.assetIdsEnabledByDefault
import Primitives
import PrimitivesTestKit
@testable import StreamService
import StreamServiceTestKit
import Testing
import WalletSessionService
import WalletSessionServiceTestKit
import WebSocketClientTestKit

struct StreamSubscriptionServiceTests {
    private let decoder = JSONDecoder()

    @Test
    func setupBeforeConnectionSubscribesOnce() async throws {
        let webSocket = WebSocketConnectionMock()
        let service = makeService(webSocket: webSocket)

        try await service.setupAssets()
        #expect(await webSocket.getSentData().isEmpty)

        await webSocket.simulateConnected()
        await service.resubscribe()

        let messages = try await sentMessages(webSocket)
        #expect(messages.count == 1)
        #expect(Set(messages.first?.assets ?? []) == Set(try assetIdsEnabledByDefault().map(AssetId.init(id:))))
    }

    @Test
    func setupSkipsSameAssets() async throws {
        let webSocket = WebSocketConnectionMock()
        let service = makeService(webSocket: webSocket)

        await webSocket.simulateConnected()
        try await service.setupAssets()
        try await service.setupAssets()
        await service.resubscribe()

        let messages = try await sentMessages(webSocket)
        #expect(messages.count == 1)
        #expect(Set(messages.first?.assets ?? []) == Set(try assetIdsEnabledByDefault().map(AssetId.init(id:))))
    }

    @Test
    func resetAllowsReconnectResubscribe() async throws {
        let webSocket = WebSocketConnectionMock()
        let service = makeService(webSocket: webSocket)

        await webSocket.simulateConnected()
        try await service.setupAssets()
        await service.resetSubscriptions()
        await service.resubscribe()

        let messages = try await sentMessages(webSocket)
        #expect(messages.count == 2)
        #expect(Set(messages.last?.assets ?? []) == Set(try assetIdsEnabledByDefault().map(AssetId.init(id:))))
    }

    @Test
    func setupSkipsWithoutCurrentWallet() async throws {
        let webSocket = WebSocketConnectionMock()
        let service = makeService(webSocket: webSocket, currentWalletId: nil)

        await webSocket.simulateConnected()
        try await service.setupAssets()
        await service.resubscribe()

        #expect(await webSocket.getSentData().isEmpty)
    }

    private func makeService(
        webSocket: WebSocketConnectionMock,
        currentWalletId: WalletId? = .mock(),
    ) -> StreamSubscriptionService {
        let walletSessionService = WalletSessionService.mock()
        walletSessionService.setCurrent(walletId: currentWalletId)
        return .mock(walletSessionService: walletSessionService, webSocket: webSocket)
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
