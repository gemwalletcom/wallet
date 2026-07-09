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
            query.request = NFTRequest(walletId: newWallet.id, filter: .all)
        }
    }

    func buildGridItem(from data: NFTData) -> GridPosterViewItem {
        NFTGridPosterBuilder.item(from: data)
    }

    func buildGridItem(collection: NFTCollection, asset: NFTAsset) -> GridPosterViewItem {
        NFTGridPosterBuilder.item(collection: collection, asset: asset)
    }
}
