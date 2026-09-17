// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
import GemstoneServices
import NFT
import Primitives
import PrimitivesComponents
import Store
import SwiftUI
import class Gemstone.GemCollectibleService

extension ViewModelFactory {
    @MainActor
    public func collectionsScene(wallet: Wallet) -> CollectionsViewModel {
        CollectionsViewModel(service: nftService, wallet: wallet)
    }

    @MainActor
    public func collectionScene(wallet: Wallet, collectionId: String) -> CollectionViewModel {
        CollectionViewModel(service: nftService, wallet: wallet, collectionId: collectionId)
    }

    @MainActor
    public func unverifiedCollectionsScene(wallet: Wallet) -> UnverifiedCollectionsViewModel {
        UnverifiedCollectionsViewModel(service: nftService, wallet: wallet)
    }

    @MainActor
    public func collectibleScene(wallet: Wallet, assetData: NFTAssetData, isPresentingSelectedAssetInput: Binding<SelectedAssetInput?>) -> CollectibleViewModel {
        CollectibleViewModel(
            wallet: wallet,
            assetData: assetData,
            service: GemCollectibleService(nfts: nftService, avatars: avatarService, explorer: explorerService),
            isPresentingSelectedAssetInput: isPresentingSelectedAssetInput,
        )
    }
}
