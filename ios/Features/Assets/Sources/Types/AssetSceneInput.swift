// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemTransactionRow
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import Store

public struct AssetSceneInput: Sendable {
    public let wallet: Wallet
    public let asset: Asset

    public var assetRequest: ChainAssetQuery
    public var transactionsRequest: MappedQuery<TransactionsQuery, [ListSection<GemTransactionRow>]>
    public var bannersRequest: BannersQuery

    public init(wallet: Wallet, asset: Asset) {
        self.wallet = wallet
        self.asset = asset

        assetRequest = ChainAssetQuery(
            walletId: wallet.id,
            assetId: asset.id,
        )

        transactionsRequest = MappedQuery(
            TransactionsQuery.assetScene(walletId: wallet.id, assetId: asset.id, limit: GemConstants.transactionsListLimit),
            transform: transactionListSections,
        )

        bannersRequest = BannersQuery(
            walletId: wallet.id,
            assetId: asset.id,
            events: BannerEvent.allCases,
        )
    }
}
