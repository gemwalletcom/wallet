// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemFormattedNumber
import GemstonePrimitives
import Localization
import Primitives

struct TransactionPriceViewModel {
    private let price: GemFormattedNumber?

    init(price: GemFormattedNumber?) {
        self.price = price
    }
}

extension TransactionPriceViewModel: ItemModelProvidable {
    var itemModel: TransactionItemModel {
        guard let price else {
            return .empty
        }
        return .price(ListItemModel(title: Localized.Asset.price, subtitle: price.text()))
    }
}
