// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemCollectibleService
import protocol Gemstone.GemCollectibleServiceProtocol
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
        assetId: NFTAssetId = .mock(),
        assetData: NFTAssetData? = .mock(),
        service: any GemCollectibleServiceProtocol = GemCollectibleService.mock(),
        gallery: any ImageGallerySaving = ImageGallerySaverMock(),
        onSelectAddress: (@MainActor @Sendable (ChainAddress) -> Void)? = nil,
    ) -> CollectibleSceneViewModel {
        CollectibleSceneViewModel(
            wallet: wallet,
            collectible: assetData.map { .assetData($0) } ?? .assetId(assetId),
            service: service,
            gallery: gallery,
            isPresentingSelectedAssetInput: .constant(.none),
            onSelectAddress: onSelectAddress,
        )
    }
}
