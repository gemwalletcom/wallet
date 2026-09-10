// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import struct Gemstone.GemConfirmMetadata
import protocol Gemstone.GemConfirmSessionProtocol
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
    private let session: any GemConfirmSessionProtocol

    init(
        type: TransactionInputType,
        metadata: GemConfirmMetadata?,
        session: any GemConfirmSessionProtocol,
    ) {
        self.type = type
        self.metadata = metadata
        self.session = session
    }
}

// MARK: - ItemModelProvidable

extension ConfirmDetailsViewModel: ItemModelProvidable {
    public var itemModel: ConfirmTransferItemModel {
        switch type {
        case let .swap(fromAsset, toAsset, swapData):
            let toAsset = toAsset.map()
            let quote = swapData.quote
            let summary = swapQuoteSummary(quote: quote)
            let fromAssetPrice = AssetPriceValue(asset: fromAsset.map(), price: metadata?.assetPrice)
            let toAssetPrice = AssetPriceValue(asset: toAsset, price: metadata?.assetPrices[toAsset.id])
            return .swapDetails(
                SwapDetailsViewModel(
                    fromAssetPrice: fromAssetPrice,
                    toAssetPrice: toAssetPrice,
                    selectedQuote: quote,
                    slippage: .manual(bps: quote.slippageBps),
                    currency: session.currency.rawValue,
                    swapPriceImpact: fromAssetPrice.swapValue(quote.fromValue)
                        .priceImpact(receive: toAssetPrice.swapValue(quote.toValue))
                        .map { $0.map() },
                    minReceiveValue: BigInt(summary.minReceiveValue),
                    etaMinutes: summary.etaMinutes,
                ),
            )
        case let .perpetual(_, perpetualType):
            if case let .modify(data) = perpetualType {
                return .perpetualModifyPosition(PerpetualModifyViewModel(summary: session.autocloseSummary(data: data)))
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
             .earn:
            return .empty
        }
    }
}
