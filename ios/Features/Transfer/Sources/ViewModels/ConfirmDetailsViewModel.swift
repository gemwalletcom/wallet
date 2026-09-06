// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import struct Gemstone.GemConfirmMetadata
import protocol Gemstone.GemConfirmTransferServiceProtocol
import enum Gemstone.TransactionInputType
import class Gemstone.GemSwapQuoteSummary
import BigInt
import Components
import Primitives
import PrimitivesComponents
import Swap

public struct ConfirmDetailsViewModel {
    private let type: TransactionInputType
    private let metadata: GemConfirmMetadata?
    private let currency: String
    private let service: any GemConfirmTransferServiceProtocol

    init(
        type: TransactionInputType,
        metadata: GemConfirmMetadata?,
        currency: String,
        service: any GemConfirmTransferServiceProtocol,
    ) {
        self.type = type
        self.metadata = metadata
        self.currency = currency
        self.service = service
    }
}

// MARK: - ItemModelProvidable

extension ConfirmDetailsViewModel: ItemModelProvidable {
    public var itemModel: ConfirmTransferItemModel {
        switch type {
        case let .swap(fromAsset, toAsset, swapData):
            let toAsset = toAsset.map()
            let quote = swapData.quote
            let summary = GemSwapQuoteSummary(quote: quote)
            let fromAssetPrice = AssetPriceValue(asset: fromAsset.map(), price: metadata?.assetPrice)
            let toAssetPrice = AssetPriceValue(asset: toAsset, price: metadata?.assetPrices[toAsset.id])
            return .swapDetails(
                SwapDetailsViewModel(
                    fromAssetPrice: fromAssetPrice,
                    toAssetPrice: toAssetPrice,
                    selectedQuote: quote,
                    slippage: .manual(bps: quote.slippageBps),
                    currency: currency,
                    swapPriceImpact: fromAssetPrice.swapValue(quote.fromValue)
                        .priceImpact(receive: toAssetPrice.swapValue(quote.toValue))
                        .map { $0.map() },
                    minReceiveValue: BigInt(summary.minReceiveValue()),
                    etaMinutes: summary.etaMinutes(),
                ),
            )
        case let .perpetual(_, perpetualType):
            let perpetualType = Primitives.PerpetualType(core: perpetualType)
            return switch perpetualType {
            case .open, .close, .increase, .reduce:
                .perpetualDetails(PerpetualDetailsViewModel(type: PerpetualDetailsType(perpetualType)))
            case let .modify(data):
                .perpetualModifyPosition(PerpetualModifyViewModel(summary: service.autocloseSummary(data: data.json())))
            }
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
