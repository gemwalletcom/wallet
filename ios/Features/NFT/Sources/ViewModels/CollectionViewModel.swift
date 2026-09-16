// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import protocol Gemstone.GemNftServiceProtocol
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import Store
import SwiftUI

@Observable
@MainActor
public final class CollectionViewModel: CollectionsViewable, Sendable {
    private let service: any GemNftServiceProtocol

    public let query: ObservableQuery<NFTRequest>

    public var isPresentingReceiveSelectAssetType: SelectAssetType?

    public init(
        service: any GemNftServiceProtocol,
        wallet: Wallet,
        collectionId: String,
    ) {
        self.service = service
        query = ObservableQuery(NFTRequest(walletId: wallet.id, filter: .collection(id: collectionId)), initialValue: [])
    }

    public var title: String {
        query.value.first?.collection.name ?? ""
    }

    public var content: CollectionsContent {
        CollectionsContent(items: NFTGridPosterBuilder.items(service.listItems(data: query.value.map { $0.toGem() }, list: .collection)))
    }
}
