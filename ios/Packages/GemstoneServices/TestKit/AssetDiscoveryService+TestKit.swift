// Copyright (c). Gem Wallet. All rights reserved.

import PrimitivesTestKit
import GemstoneServices
import protocol Gemstone.GemAssetDiscoveryServiceProtocol
import GemstonePrimitivesTestKit
import Preferences
import PreferencesTestKit

public extension AssetDiscoverable where Self == AssetDiscoveryService {
    static func mock(
        discovery: any GemAssetDiscoveryServiceProtocol = GemAssetDiscoveryServiceMock(),
        preferences: Preferences = .mock(),
    ) -> AssetDiscoveryService {
        AssetDiscoveryService(discovery: discovery, preferences: preferences)
    }
}
