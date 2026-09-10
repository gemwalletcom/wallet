// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemNftServiceProtocol
import GemstonePrimitives
import Localization
import Primitives
import Store
import SwiftUI

@Observable
@MainActor
public final class UnverifiedCollectionsViewModel: CollectionsViewable, Sendable {
    private let service: any GemNftServiceProtocol

    public let query: ObservableQuery<NFTRequest>

    public var isPresentingReceiveSelectAssetType: SelectAssetType?

    public init(service: any GemNftServiceProtocol, wallet: Wallet) {
        self.service = service
        query = ObservableQuery(NFTRequest(walletId: wallet.id, filter: .unverified), initialValue: [])
    }

    public var title: String {
        Localized.Asset.Verification.unverified
    }

    public var content: CollectionsContent {
        CollectionsContent(items: service.listItems(data: query.value.map { $0.map() }, list: .unverified).map(NFTGridPosterBuilder.item))
    }
}
