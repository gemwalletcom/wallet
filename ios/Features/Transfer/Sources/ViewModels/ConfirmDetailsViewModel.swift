// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import struct Gemstone.GemConfirmMetadata
import protocol Gemstone.GemConfirmTransferServiceProtocol
import enum Gemstone.GemTransactionInputType
import Components
import Primitives
import PrimitivesComponents
import Swap

public struct ConfirmDetailsViewModel {
    private let type: GemTransactionInputType
    private let metadata: GemConfirmMetadata?
    private let currency: String
    private let service: any GemConfirmTransferServiceProtocol

    init(
        type: GemTransactionInputType,
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
            let quote = Primitives.SwapData(core: swapData).quote
            let fromAssetPrice = AssetPriceValue(asset: fromAsset.map(), price: metadata?.assetPrice)
            let toAssetPrice = AssetPriceValue(asset: toAsset, price: metadata?.assetPrices[toAsset.id])
            return .swapDetails(
                SwapDetailsViewModel(
                    fromAssetPrice: fromAssetPrice,
                    toAssetPrice: toAssetPrice,
                    selectedQuote: quote,
                    slippage: .manual(bps: quote.slippageBps),
                    currency: currency,
                    swapPriceImpact: service.swapPriceImpact(
                        pay: fromAssetPrice.swapValue(quote.fromValue),
                        receive: toAssetPrice.swapValue(quote.toValue),
                    ).flatMap { try? Primitives.SwapPriceImpact($0) },
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
