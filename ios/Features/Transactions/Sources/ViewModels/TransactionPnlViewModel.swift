// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import Localization
import Primitives
import Style
import SwiftUI

struct TransactionPnlViewModel {
    private let pnl: Double?
    private let currencyFormatter: CurrencyFormatter

    init(pnl: Double?, currencyFormatter: CurrencyFormatter = .usd) {
        self.pnl = pnl
        self.currencyFormatter = currencyFormatter
    }
}

extension TransactionPnlViewModel: ItemModelProvidable {
    var itemModel: TransactionItemModel {
        guard let pnl else {
            return .empty
        }

        let sign = pnl >= 0 ? "+" : ""
        let pnlFormatted = currencyFormatter.string(pnl)
        let color = pnl >= 0 ? Colors.green : Colors.red

        return .pnl(
            title: Localized.Perpetual.pnl,
            value: "\(sign)\(pnlFormatted)",
            color: color,
        )
    }
}
