// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemPriceServiceProtocol
import Foundation
import GemstoneServices
import Primitives
import WebSocketClient

public actor StreamSubscriptionService: Sendable {
    private let priceService: any GemPriceServiceProtocol
    private let walletSessionService: any WalletSessionManageable
    private let webSocket: any WebSocketConnectable
    private let encoder = JSONEncoder()

    private var subscribedAssetIds: Set<AssetId> = []

    public init(
        priceService: any GemPriceServiceProtocol,
        walletSessionService: any WalletSessionManageable,
        webSocket: any WebSocketConnectable,
    ) {
        self.priceService = priceService
        self.walletSessionService = walletSessionService
        self.webSocket = webSocket
    }

    public func setupAssets() async throws {
        guard let walletId = walletSessionService.currentWalletId else { return }
        guard await webSocket.state == .connected else { return }

        let assets = try await priceService.observableAssetIds(walletId: walletId.id).map { try AssetId(id: $0) }
        let assetIds = Set(assets)
        guard subscribedAssetIds != assetIds else { return }

        let message = StreamMessage.subscribePrices(StreamMessagePrices(assets: assets))
        try await sendMessage(message)
        subscribedAssetIds = assetIds
    }

    public func resubscribe() async {
        do {
            try await setupAssets()
        } catch {
            debugLog("stream subscription: resubscribe failed: \(error)")
        }
    }

    func resetSubscriptions() {
        subscribedAssetIds.removeAll()
    }

    private func sendMessage(_ message: StreamMessage) async throws {
        let data = try encoder.encode(message)
        try await webSocket.send(data)
        debugLog("stream subscription send message: \(message)")
    }
}

// MARK: - PriceUpdater

extension StreamSubscriptionService: PriceUpdater {
    public func addPrices(assetIds: [AssetId]) async throws {
        let newAssets = Set(assetIds).subtracting(subscribedAssetIds).asArray()
        guard newAssets.isNotEmpty else { return }

        try await sendMessage(StreamMessage.addPrices(StreamMessagePrices(assets: newAssets)))
        subscribedAssetIds.formUnion(newAssets)
    }
}
