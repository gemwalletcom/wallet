// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import Localization
import Primitives
import Style
import SwiftUI

struct TransactionPnlViewModel {
    private let metadata: TransactionPerpetualMetadata?
    private let currencyFormatter: CurrencyFormatter

    init(metadata: TransactionPerpetualMetadata?, currencyFormatter: CurrencyFormatter = .usd) {
        self.metadata = metadata
        self.currencyFormatter = currencyFormatter
    }
}

extension TransactionPnlViewModel: ItemModelProvidable {
    var itemModel: TransactionItemModel {
        guard let metadata, metadata.pnl != 0 else {
            return .empty
        }

        let sign = metadata.pnl >= 0 ? "+" : ""
        let pnlFormatted = currencyFormatter.string(metadata.pnl)
        let color = metadata.pnl >= 0 ? Colors.green : Colors.red

        return .pnl(
            title: Localized.Perpetual.pnl,
            value: "\(sign)\(pnlFormatted)",
            color: color,
        )
    }
}
