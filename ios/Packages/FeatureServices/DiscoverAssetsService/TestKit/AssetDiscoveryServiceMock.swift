// Copyright (c). Gem Wallet. All rights reserved.

import DiscoverAssetsService
import Foundation
import Primitives

public struct AssetDiscoveryServiceMock: AssetDiscoverable {
    public init() {}

    public func discoverAssets(wallet _: Wallet) async throws {}
}

public extension AssetDiscoverable where Self == AssetDiscoveryServiceMock {
    static func mock() -> AssetDiscoveryServiceMock {
        AssetDiscoveryServiceMock()
    }
}
