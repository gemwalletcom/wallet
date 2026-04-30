// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Primitives
import PrimitivesComponents
import Store
import SwiftUI

@Observable
@MainActor
public final class CollectionViewModel: CollectionsViewable, Sendable {
    private let collectionName: String

    public let query: ObservableQuery<NFTRequest>
    public var nftDataList: [NFTData] {
        query.value
    }

    public var isPresentingReceiveSelectAssetType: SelectAssetType?

    public var wallet: Wallet

    public init(
        wallet: Wallet,
        collectionId: String,
        collectionName: String,
    ) {
        self.wallet = wallet
        self.collectionName = collectionName
        query = ObservableQuery(NFTRequest(walletId: wallet.walletId, filter: .collection(id: collectionId)), initialValue: [])
    }

    public var title: String {
        collectionName
    }

    public var content: CollectionsContent {
        CollectionsContent(
            items: nftDataList.flatMap { data in
                data.assets.map { buildGridItem(collection: data.collection, asset: $0) }
            },
        )
    }
}
