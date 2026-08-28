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

    var title: String { get }
    var columns: [GridItem] { get }
    var content: CollectionsContent { get }
    var emptyContentModel: EmptyContentTypeViewModel { get }

    var isPresentingReceiveSelectAssetType: SelectAssetType? { get set }

    func load() async
    func onSelectReceive()
}

public extension CollectionsViewable {
    var columns: [GridItem] {
        Array(repeating: GridItem(spacing: .medium), count: 2)
    }

    var emptyContentModel: EmptyContentTypeViewModel {
        EmptyContentTypeViewModel(type: .nfts(action: onSelectReceive))
    }

    func load() async {}

    func onSelectReceive() {
        isPresentingReceiveSelectAssetType = .receive(.collection)
    }

    func buildGridItem(from data: NFTData) -> GridPosterViewItem {
        NFTGridPosterBuilder.item(from: data)
    }

    func buildGridItem(collection: NFTCollection, asset: NFTAsset) -> GridPosterViewItem {
        NFTGridPosterBuilder.item(collection: collection, asset: asset)
    }
}
