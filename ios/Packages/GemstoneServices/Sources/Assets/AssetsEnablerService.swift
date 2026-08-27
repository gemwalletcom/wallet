// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import typealias Gemstone.Currency
import protocol Gemstone.GemBalanceServiceProtocol
import GemstonePrimitives
import Preferences
import Primitives

public struct AssetsEnablerService: AssetsEnabler {
    private let service: any GemBalanceServiceProtocol
    private let preferences: Preferences

    public init(
        service: any GemBalanceServiceProtocol,
        preferences: Preferences,
    ) {
        self.service = service
        self.preferences = preferences
    }

    public func enableAssets(wallet: Wallet, assetIds: [AssetId], enabled: Bool) async throws {
        try await service.enableAssets(walletId: wallet.id.id, assetIds: assetIds.ids, enabled: enabled, currency: currency())
    }

    public func pinAsset(wallet: Wallet, assetId: AssetId, pinned: Bool) async throws {
        try await service.pinAsset(walletId: wallet.id.id, assetId: assetId.identifier, pinned: pinned, currency: currency())
    }

    private func currency() throws -> Gemstone.Currency {
        try Primitives.Currency(id: preferences.currency).json()
    }
}
