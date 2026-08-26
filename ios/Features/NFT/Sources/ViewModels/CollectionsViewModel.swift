// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Localization
import GemstoneServices
import Primitives
import PrimitivesComponents
import Store
import SwiftUI

@Observable
@MainActor
public final class CollectionsViewModel: CollectionsViewable, Sendable {
    private let nftService: NFTService

    public let query: ObservableQuery<NFTRequest>

    public var isPresentingReceiveSelectAssetType: SelectAssetType?

    public let wallet: Wallet

    public init(
        nftService: NFTService,
        wallet: Wallet,
    ) {
        self.nftService = nftService
        self.wallet = wallet
        query = ObservableQuery(NFTRequest(walletId: wallet.id, filter: .all), initialValue: [])
    }

    public var title: String {
        Localized.Nft.collections
    }

    public var content: CollectionsContent {
        CollectionsContent(
            items: verifiedItems,
            unverifiedCount: unverifiedCount,
        )
    }

    // MARK: - Private

    private var nftDataList: [NFTData] {
        query.value
    }

    private var verifiedItems: [GridPosterViewItem] {
        nftDataList
            .filter { $0.collection.status == .verified }
            .map { buildGridItem(from: $0) }
    }

    private var unverifiedCount: String? {
        let unverified = nftDataList.filter { $0.collection.status != .verified }
        guard unverified.isNotEmpty else { return nil }
        return unverified.count.asString
    }

    // MARK: - Actions

    public func fetch() async {
        do {
            let count = try await nftService.updateAssets(wallet: wallet)
            debugLog("update nfts: \(count)")
        } catch {
            debugLog("update nfts error: \(error)")
        }
    }
}
