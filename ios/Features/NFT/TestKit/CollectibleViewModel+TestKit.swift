// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemCollectibleService
import GemstoneServicesTestKit
import NFT
import Primitives
import PrimitivesTestKit

public extension CollectibleViewModel {
    @MainActor
    static func mock(
        wallet: Wallet = .mock(),
        assetData: NFTAssetData = .mock(),
    ) -> CollectibleViewModel {
        CollectibleViewModel(
            wallet: wallet,
            assetData: assetData,
            service: GemCollectibleService.mock(),
            isPresentingSelectedAssetInput: .constant(.none),
        )
    }
}
