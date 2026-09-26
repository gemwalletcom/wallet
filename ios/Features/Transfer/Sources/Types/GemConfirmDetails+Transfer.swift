// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemConfirmDetails
import PrimitivesComponents
import Swap

extension GemConfirmDetails {
    var itemModel: ConfirmTransferItemModel {
        switch self {
        case let .swap(details): .swapDetails(SwapDetailsViewModel(details: details, allowSelectProvider: false))
        case let .perpetual(details): .perpetualDetails(PerpetualDetailsViewModel(details: details))
        case let .perpetualAutoclose(row): .perpetualModifyPosition(row)
        }
    }
}
