// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemNftServiceProtocol
import Components
import class Gemstone.GemNftRulesService
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
    private let nftRules = GemNftRulesService()

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
        guard let data = try? nftDataList.map({ try $0.json() }) else { return [] }
        let collections = verified ? nftRules.verifiedCollections(data: data) : nftRules.unverifiedCollections(data: data)
        return nftRules.sortedCollections(data: collections).compactMap { try? NFTData($0) }
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
