// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import func Gemstone.transactionsListLimit
import Primitives
import PrimitivesComponents
import Store

public struct AssetSceneInput: Sendable {
    public let wallet: Wallet
    public let asset: Asset

    public var assetRequest: ChainAssetRequest
    public var transactionsRequest: MappedRequest<TransactionsRequest, [ListSection<TransactionViewModel>]>
    public var bannersRequest: BannersRequest

    public init(wallet: Wallet, asset: Asset) {
        self.wallet = wallet
        self.asset = asset

        assetRequest = ChainAssetRequest(
            walletId: wallet.id,
            assetId: asset.id,
        )

        transactionsRequest = MappedRequest(
            TransactionsRequest.assetScene(walletId: wallet.id, assetId: asset.id, limit: Int(transactionsListLimit())),
            transform: TransactionViewModel.sections,
        )

        bannersRequest = BannersRequest(
            walletId: wallet.id,
            assetId: asset.id,
            events: BannerEvent.allCases,
        )
    }
}
