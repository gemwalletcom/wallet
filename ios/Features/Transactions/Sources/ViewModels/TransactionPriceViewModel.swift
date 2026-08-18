// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import Localization
import Primitives

struct TransactionPriceViewModel {
    private let metadata: TransactionPerpetualMetadata?
    private let currencyFormatter: CurrencyFormatter

    init(metadata: TransactionPerpetualMetadata?, currencyFormatter: CurrencyFormatter = .usd) {
        self.metadata = metadata
        self.currencyFormatter = currencyFormatter
    }
}

extension TransactionPriceViewModel: ItemModelProvidable {
    var itemModel: TransactionItemModel {
        guard let metadata, metadata.price > 0 else {
            return .empty
        }

        let priceFormatted = currencyFormatter.string(metadata.price)

        return .price(
            title: Localized.Asset.price,
            value: priceFormatted,
        )
    }
}
