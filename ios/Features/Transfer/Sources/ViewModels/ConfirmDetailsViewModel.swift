// Copyright (c). Gem Wallet. All rights reserved.

import Components
import class Gemstone.GemSwapQuoteService
import protocol Gemstone.GemPerpetualServiceProtocol
import Primitives
import PrimitivesComponents
import Swap

public struct ConfirmDetailsViewModel {
    private let type: TransferDataType
    private let metadata: TransferDataMetadata?
    private let currency: String
    private let perpetualService: any GemPerpetualServiceProtocol
    private let swapQuoteService: GemSwapQuoteService

    init(
        type: TransferDataType,
        metadata: TransferDataMetadata?,
        currency: String,
        perpetualService: any GemPerpetualServiceProtocol,
        swapQuoteService: GemSwapQuoteService,
    ) {
        self.type = type
        self.metadata = metadata
        self.currency = currency
        self.perpetualService = perpetualService
        self.swapQuoteService = swapQuoteService
    }
}

// MARK: - ItemModelProvidable

extension ConfirmDetailsViewModel: ItemModelProvidable {
    public var itemModel: ConfirmTransferItemModel {
        switch type {
        case let .swap(fromAsset, toAsset, swapData):
            .swapDetails(
                SwapDetailsViewModel(
                    fromAssetPrice: AssetPriceValue(asset: fromAsset, price: metadata?.assetPrice),
                    toAssetPrice: AssetPriceValue(asset: toAsset, price: metadata?.assetPrices[toAsset.id]),
                    selectedQuote: swapData.quote,
                    slippage: .manual(bps: swapData.quote.slippageBps),
                    currency: currency,
                    swapQuoteService: swapQuoteService,
                ),
            )
        case let .perpetual(_, perpetualType):
            switch perpetualType {
            case .open, .close, .increase, .reduce:
                .perpetualDetails(PerpetualDetailsViewModel(type: PerpetualDetailsType(perpetualType)))
            case let .modify(data):
                .perpetualModifyPosition(PerpetualModifyViewModel(data: data, perpetualService: perpetualService))
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
            .empty
        }
    }
}
