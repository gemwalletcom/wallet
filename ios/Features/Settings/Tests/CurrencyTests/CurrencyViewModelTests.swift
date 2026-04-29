// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
@testable import Settings
import Testing

struct CurrencyViewModelTests {
    @Test
    func uSTitle() throws {
        let currency = try #require(Currency(rawValue: "USD"))
        let viewModel = CurrencyViewModel(currency: currency)
        #expect(viewModel.title == "🇺🇸 USD - US Dollar")
    }

    @Test
    func eUROTitle() throws {
        let currency = try #require(Currency(rawValue: "EUR"))
        let viewModel = CurrencyViewModel(currency: currency)
        #expect(viewModel.title == "🇪🇺 EUR - Euro")
    }
}
