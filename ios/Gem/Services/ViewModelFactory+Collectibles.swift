// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemCollectibleService
import GemstonePrimitives
import GemstoneServices
import NFT
import Primitives
import PrimitivesComponents
import Store
import SwiftUI

public extension ViewModelFactory {
    @MainActor
    func collectionsScene(wallet: Wallet) -> CollectionsViewModel {
        CollectionsViewModel(service: nftService, wallet: wallet, list: .collections)
    }

    @MainActor
    func collectionScene(wallet: Wallet, collectionId: String) -> CollectionsViewModel {
        CollectionsViewModel(service: nftService, wallet: wallet, list: .collection, collectionId: collectionId)
    }

    @MainActor
    func unverifiedCollectionsScene(wallet: Wallet) -> CollectionsViewModel {
        CollectionsViewModel(service: nftService, wallet: wallet, list: .unverified)
    }

    @MainActor
    func collectibleScene(
        wallet: Wallet,
        assetData: NFTAssetData,
        isPresentingSelectedAssetInput: Binding<SelectedAssetInput?>,
        onSelectAddress: @escaping @MainActor @Sendable (ChainAddress) -> Void,
    ) -> CollectibleViewModel {
        CollectibleViewModel(
            wallet: wallet,
            assetData: assetData,
            service: GemCollectibleService(nfts: nftService, avatars: avatarService, explorer: explorerService),
            isPresentingSelectedAssetInput: isPresentingSelectedAssetInput,
            onSelectAddress: onSelectAddress,
        )
    }
}
