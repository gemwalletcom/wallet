// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import struct Gemstone.GemConfirmMetadata
import protocol Gemstone.GemConfirmSceneServiceProtocol
import Components
import class Gemstone.GemSwapQuoteService
import Primitives
import PrimitivesComponents
import Swap

public struct ConfirmDetailsViewModel {
    private let type: TransferDataType
    private let metadata: GemConfirmMetadata?
    private let currency: String
    private let service: any GemConfirmSceneServiceProtocol

    init(
        type: TransferDataType,
        metadata: GemConfirmMetadata?,
        currency: String,
        service: any GemConfirmSceneServiceProtocol,
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
            .swapDetails(
                SwapDetailsViewModel(
                    fromAssetPrice: AssetPriceValue(asset: fromAsset, price: metadata?.assetPrice),
                    toAssetPrice: AssetPriceValue(asset: toAsset, price: metadata?.assetPrices[toAsset.id]),
                    selectedQuote: swapData.quote,
                    slippage: .manual(bps: swapData.quote.slippageBps),
                    currency: currency,
                    swapQuoteService: service.swapQuote(),
                ),
            )
        case let .perpetual(_, perpetualType):
            switch perpetualType {
            case .open, .close, .increase, .reduce:
                .perpetualDetails(PerpetualDetailsViewModel(type: PerpetualDetailsType(perpetualType)))
            case let .modify(data):
                .perpetualModifyPosition(PerpetualModifyViewModel(data: data, service: service))
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
