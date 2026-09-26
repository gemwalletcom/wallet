// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemCollectibleService
import GemstoneServicesTestKit
import ImageGalleryService
import ImageGalleryServiceTestKit
import NFT
import Primitives
import PrimitivesTestKit

public extension CollectibleSceneViewModel {
    @MainActor
    static func mock(
        wallet: Wallet = .mock(),
        assetData: NFTAssetData = .mock(),
        gallery: any ImageGallerySaving = ImageGallerySaverMock(),
        onSelectAddress: (@MainActor @Sendable (ChainAddress) -> Void)? = nil,
    ) -> CollectibleSceneViewModel {
        CollectibleSceneViewModel(
            wallet: wallet,
            assetData: assetData,
            service: GemCollectibleService.mock(),
            gallery: gallery,
            isPresentingSelectedAssetInput: .constant(.none),
            onSelectAddress: onSelectAddress,
        )
    }
}
