// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import struct Gemstone.GemConfirmMetadata
import protocol Gemstone.GemConfirmationProtocol
import enum Gemstone.TransactionInputType
import struct Gemstone.GemSwapQuoteSummary
import func Gemstone.perpetualDetails
import func Gemstone.swapQuoteSummary
import BigInt
import Components
import Primitives
import PrimitivesComponents
import Swap

public struct ConfirmDetailsViewModel {
    private let type: TransactionInputType
    private let metadata: GemConfirmMetadata?
    private let confirmation: any GemConfirmationProtocol

    init(
        type: TransactionInputType,
        metadata: GemConfirmMetadata?,
        confirmation: any GemConfirmationProtocol,
    ) {
        self.type = type
        self.metadata = metadata
        self.confirmation = confirmation
    }
}

// MARK: - ItemModelProvidable

extension ConfirmDetailsViewModel: ItemModelProvidable {
    public var itemModel: ConfirmTransferItemModel {
        switch type {
        case let .swap(fromAsset, toAsset, swapData):
            let quote = swapData.quote
            let summary = swapQuoteSummary(quote: quote, fromAsset: fromAsset, toAsset: toAsset)
            let toAsset = toAsset.toPrimitives()
            let fromAssetPrice = AssetPriceValue(asset: fromAsset.toPrimitives(), price: metadata?.assetPrice)
            let toAssetPrice = AssetPriceValue(asset: toAsset, price: metadata?.assetPrices[toAsset.id])
            return .swapDetails(
                SwapDetailsViewModel(
                    fromAssetPrice: fromAssetPrice,
                    toAssetPrice: toAssetPrice,
                    summary: summary,
                    slippagePercent: summary.slippagePercent(),
                    currency: confirmation.currency.rawValue,
                    swapPriceImpact: fromAssetPrice.swapValue(quote.fromValue)
                        .priceImpact(receive: toAssetPrice.swapValue(quote.toValue)),
                ),
            )
        case let .perpetual(_, perpetualType):
            if case let .modify(data) = perpetualType {
                return .perpetualModifyPosition(PerpetualModifyViewModel(summary: confirmation.autocloseSummary(data: data)))
            }
            guard let details = perpetualDetails(perpetualType: perpetualType) else { return .empty }
            return .perpetualDetails(PerpetualDetailsViewModel(details: details))
        case .transfer,
             .deposit,
             .withdrawal,
             .transferNft,
             .tokenApprove,
             .stake,
             .account,
             .generic,
             .payment,
             .earn:
            return .empty
        }
    }
}
