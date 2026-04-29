// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import Localization
import Primitives

struct TransactionPriceViewModel {
    private let metadata: TransactionPerpetualMetadata?

    init(metadata: TransactionPerpetualMetadata?) {
        self.metadata = metadata
    }
}

extension TransactionPriceViewModel: ItemModelProvidable {
    var itemModel: TransactionItemModel {
        guard let metadata, metadata.price > 0 else {
            return .empty
        }

        let formatter = CurrencyFormatter(type: .currency, currencyCode: Currency.usd.rawValue)
        let priceFormatted = formatter.string(metadata.price)

        return .price(
            title: Localized.Asset.price,
            value: priceFormatted,
        )
    }
}
