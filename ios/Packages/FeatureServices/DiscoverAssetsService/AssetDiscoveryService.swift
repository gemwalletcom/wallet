// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemAssetDiscoveryServiceProtocol
import GemstonePrimitives
import Preferences
import Primitives

public struct AssetDiscoveryService: AssetDiscoverable {
    private let discovery: any GemAssetDiscoveryServiceProtocol
    private let preferences: Preferences

    public init(
        discovery: any GemAssetDiscoveryServiceProtocol,
        preferences: Preferences,
    ) {
        self.discovery = discovery
        self.preferences = preferences
    }

    public func discoverAssets(wallet: Wallet) async throws {
        _ = try await discovery.discover(walletId: wallet.id.id, currency: Currency(id: preferences.currency).json())
    }
}
