// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Primitives
import PrimitivesComponents
import Store
import SwiftUI

@MainActor
public protocol CollectionsViewable: AnyObject, Observable {
    var query: ObservableQuery<NFTRequest> { get }
    var nftDataList: [NFTData] { get }

    var title: String { get }
    var columns: [GridItem] { get }
    var content: CollectionsContent { get }
    var emptyContentModel: EmptyContentTypeViewModel { get }

    var wallet: Wallet { get set }

    var isPresentingReceiveSelectAssetType: SelectAssetType? { get set }

    func fetch() async
    func onChangeWallet(_ oldWallet: Wallet?, _ newWallet: Wallet?)
    func onSelectReceive()
}

public extension CollectionsViewable {
    var columns: [GridItem] {
        Array(repeating: GridItem(spacing: .medium), count: 2)
    }

    var emptyContentModel: EmptyContentTypeViewModel {
        EmptyContentTypeViewModel(type: .nfts(action: onSelectReceive))
    }

    func fetch() async {}

    func onSelectReceive() {
        isPresentingReceiveSelectAssetType = .receive(.collection)
    }

    func onChangeWallet(_: Wallet?, _ newWallet: Wallet?) {
        if let newWallet, wallet != newWallet {
            wallet = newWallet
            query.request = NFTRequest(walletId: newWallet.walletId, filter: .all)
        }
    }

    func buildGridItem(from data: NFTData) -> GridPosterViewItem {
        if data.assets.count == 1, let asset = data.assets.first {
            return buildGridItem(collection: data.collection, asset: asset)
        }
        return GridPosterViewItem(
            id: data.id,
            destination: Scenes.Collection(id: data.collection.id, name: data.collection.name),
            model: GridPosterViewModel(
                assetImage: AssetImage(type: data.collection.name, imageURL: data.collection.images.preview.url.asURL),
                title: data.collection.name,
                count: data.assets.count,
                isVerified: data.collection.status == .verified,
            ),
        )
    }

    func buildGridItem(collection: NFTCollection, asset: NFTAsset) -> GridPosterViewItem {
        GridPosterViewItem(
            id: asset.id,
            destination: Scenes.Collectible(assetData: NFTAssetData(collection: collection, asset: asset)),
            model: GridPosterViewModel(
                assetImage: AssetImage(type: collection.name, imageURL: asset.images.preview.url.asURL),
                title: asset.name,
                isVerified: collection.status == .verified,
            ),
        )
    }
}
