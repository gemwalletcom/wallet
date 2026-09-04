// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import Localization
import Primitives

struct TransactionPriceViewModel {
    private let price: Double?
    private let currencyFormatter: CurrencyFormatter

    init(price: Double?, currencyFormatter: CurrencyFormatter = .usd) {
        self.price = price
        self.currencyFormatter = currencyFormatter
    }
}

extension TransactionPriceViewModel: ItemModelProvidable {
    var itemModel: TransactionItemModel {
        guard let price else {
            return .empty
        }

        return .price(
            title: Localized.Asset.price,
            value: currencyFormatter.string(price),
        )
    }
}
