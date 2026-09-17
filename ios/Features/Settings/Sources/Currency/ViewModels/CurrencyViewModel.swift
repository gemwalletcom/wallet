// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemCurrencyRow
import GemstonePrimitives
import Primitives

struct CurrencyViewModel {
    let row: GemCurrencyRow

    init(row: GemCurrencyRow) {
        self.row = row
    }

    var currency: Currency {
        row.currency.toPrimitives()
    }

    var title: String {
        row.title(localizedName: Locale.current.localizedString(forCurrencyCode: id) ?? .empty)
    }
}

// MARK: - Identifiable

extension CurrencyViewModel: Identifiable {
    var id: String {
        currency.rawValue
    }
}
