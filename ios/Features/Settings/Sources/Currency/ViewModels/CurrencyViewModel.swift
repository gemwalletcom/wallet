// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemCurrencyRow
import GemstonePrimitives
import Primitives

struct CurrencyViewModel {
    let currency: Currency
    let flag: String

    init(row: GemCurrencyRow) {
        currency = Currency(core: row.currency)
        flag = row.flag
    }

    var title: String {
        let localizedName = Locale.current.localizedString(forCurrencyCode: id) ?? .empty
        return "\(flag) \(id) - \(localizedName)"
    }
}

// MARK: - Identifiable

extension CurrencyViewModel: Identifiable {
    var id: String {
        currency.rawValue
    }
}
