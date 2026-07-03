// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Localization
import NFTService
import Primitives
import PrimitivesComponents
import Store
import SwiftUI
import WalletService

@Observable
@MainActor
public final class CollectionsViewModel: CollectionsViewable, Sendable {
    private let walletService: WalletService
    private let nftService: NFTService

    public var wallet: Wallet
    public let query: ObservableQuery<NFTRequest>
    public var nftDataList: [NFTData] { query.value }

    public var isPresentingReceiveSelectAssetType: SelectAssetType?

    public init(
        nftService: NFTService,
        walletService: WalletService,
        wallet: Wallet,
    ) {
        self.nftService = nftService
        self.walletService = walletService
        self.wallet = wallet
        query = ObservableQuery(NFTRequest(walletId: wallet.id, filter: .all), initialValue: [])
    }

    public var title: String { Localized.Nft.collections }

    public var content: CollectionsContent {
        CollectionsContent(
            items: verifiedData.map { buildGridItem(from: $0) },
            unverifiedCount: unverifiedCount,
        )
    }

    public var unverifiedCount: String? {
        let unverified = nftDataList.filter { $0.collection.status != .verified }
        guard unverified.isNotEmpty else { return nil }
        return unverified.count.asString
    }

    public var isEmpty: Bool {
        content.items.isEmpty && unverifiedCount == nil
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

    // MARK: - Private

    private var verifiedData: [NFTData] {
        nftDataList.filter { $0.collection.status == .verified }
    }
}
