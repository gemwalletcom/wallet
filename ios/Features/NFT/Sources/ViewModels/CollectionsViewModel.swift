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
public final class CollectionsViewModel: CollectionsViewable, Sendable {
    private let service: any GemNftServiceProtocol

    public let query: ObservableQuery<NFTRequest>

    public var isPresentingReceiveSelectAssetType: SelectAssetType?

    public let wallet: Wallet

    public init(
        service: any GemNftServiceProtocol,
        wallet: Wallet,
    ) {
        self.service = service
        self.wallet = wallet
        query = ObservableQuery(NFTRequest(walletId: wallet.id, filter: .all), initialValue: [])
    }

    public var title: String {
        Localized.Nft.collections
    }

    public var content: CollectionsContent {
        let data = query.value.map { $0.toGem() }
        return CollectionsContent(
            items: NFTGridPosterBuilder.items(service.listItems(data: data, list: .collections)),
            unverifiedCount: service.unverifiedRow(data: data, list: .collections)?.countText,
        )
    }

    // MARK: - Actions

    public func load() async {
        do {
            let count = try await service.sync()
            debugLog("update nfts: \(count)")
        } catch {
            debugLog("update nfts error: \(error)")
        }
    }
}
