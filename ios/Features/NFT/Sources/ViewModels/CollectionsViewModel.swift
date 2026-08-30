// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemNftServiceProtocol
import Components
import Foundation
import Localization
import GemstonePrimitives
import GemstoneServices
import Primitives
import PrimitivesComponents
import Store
import SwiftUI

@Observable
@MainActor
public final class CollectionsViewModel: CollectionsViewable, Sendable {
    private let nftService: any GemNftServiceProtocol

    public let query: ObservableQuery<NFTRequest>

    public var isPresentingReceiveSelectAssetType: SelectAssetType?

    public let wallet: Wallet

    public init(
        nftService: any GemNftServiceProtocol,
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
        collections(verified: true).map { buildGridItem(from: $0) }
    }

    private var unverifiedCount: String? {
        let unverified = collections(verified: false)
        guard unverified.isNotEmpty else { return nil }
        return unverified.count.asString
    }

    private func collections(verified: Bool) -> [NFTData] {
        let data = nftDataList.map { $0.json() }
        let collections = verified ? nftService.verifiedCollections(data: data) : nftService.unverifiedCollections(data: data)
        return nftService.sortedCollections(data: collections).compactMap { try? NFTData($0) }
    }

    // MARK: - Actions

    public func load() async {
        do {
            let count = try await nftService.sync(walletId: wallet.id.id)
            debugLog("update nfts: \(count)")
        } catch {
            debugLog("update nfts error: \(error)")
        }
    }
}
