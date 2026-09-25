// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import protocol Gemstone.GemConfirmationProtocol
import struct Gemstone.GemConfirmMetadata
import struct Gemstone.GemSwapQuoteSummary
import func Gemstone.perpetualConfirmDetails
import func Gemstone.swapQuoteSummary
import enum Gemstone.TransactionInputType
import GemstonePrimitives
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
            let fromAssetPrice = AssetPriceValue(asset: fromAsset.toPrimitives(), price: metadata?.assetPrice)
            let toAssetPrice = AssetPriceValue(asset: toAsset.toPrimitives(), price: metadata?.price(for: toAsset.id))
            let summary = swapQuoteSummary(quote: quote, fromAsset: fromAsset, toAsset: toAsset, fromPrice: fromAssetPrice.price?.price, toPrice: toAssetPrice.price?.price)
            return .swapDetails(
                SwapDetailsViewModel(
                    fromAssetPrice: fromAssetPrice,
                    toAssetPrice: toAssetPrice,
                    summary: summary,
                    slippagePercent: summary.slippagePercent(),
                    currency: confirmation.currency.rawValue,
                    allowSelectProvider: false,
                ),
            )
        case let .perpetual(_, perpetualType):
            if case let .modify(data) = perpetualType {
                return .perpetualModifyPosition(confirmation.autocloseRow(data: data))
            }
            guard let details = perpetualConfirmDetails(perpetualType: perpetualType) else { return .empty }
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
