// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemCurrencyRow
import Primitives
@testable import Settings
import Testing

struct CurrencyViewModelTests {
    @Test
    func uSTitle() throws {
        let viewModel = CurrencyViewModel(row: GemCurrencyRow(currency: "USD", flag: "🇺🇸"))
        #expect(viewModel.title == "🇺🇸 USD - US Dollar")
    }

    @Test
    func eUROTitle() throws {
        let viewModel = CurrencyViewModel(row: GemCurrencyRow(currency: "EUR", flag: "🇪🇺"))
        #expect(viewModel.title == "🇪🇺 EUR - Euro")
    }
}
