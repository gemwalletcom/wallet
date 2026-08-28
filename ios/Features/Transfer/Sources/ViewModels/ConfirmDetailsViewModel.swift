// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import PrimitivesComponents
import Swap

public struct ConfirmDetailsViewModel {
    private let type: TransferDataType
    private let metadata: TransferDataMetadata?
    private let currency: String

    init(type: TransferDataType, metadata: TransferDataMetadata?, currency: String) {
        self.type = type
        self.metadata = metadata
        self.currency = currency
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
                ),
            )
        case let .perpetual(_, perpetualType):
            switch perpetualType {
            case .open, .close, .increase, .reduce:
                .perpetualDetails(PerpetualDetailsViewModel(type: PerpetualDetailsType(perpetualType)))
            case let .modify(data):
                .perpetualModifyPosition(PerpetualModifyViewModel(data: data))
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
