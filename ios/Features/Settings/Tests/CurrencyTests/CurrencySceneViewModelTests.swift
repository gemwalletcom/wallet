// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import PriceServiceTestKit
import Primitives
@testable import Settings
import Testing

private final class MockCurrencyStorage: CurrencyStorable, @unchecked Sendable {
    var currency: String
    init(currency: String = "USD") {
        self.currency = currency
    }
}

struct CurrencySceneViewModelTests {
    private var storage = MockCurrencyStorage()

    @Test
    func uSDCurrencyValue() {
        let usdCurrencyStorage = MockCurrencyStorage()
        let viewModel = CurrencySceneViewModel(currencyStorage: usdCurrencyStorage, priceService: .mock())

        #expect(viewModel.selectedCurrencyValue == "🇺🇸 USD")
    }

    @Test
    func gBPCurrencyValue() {
        let gbpCurrancyStorage = MockCurrencyStorage(currency: "GBP")
        let viewModel = CurrencySceneViewModel(currencyStorage: gbpCurrancyStorage, priceService: .mock())
        #expect(viewModel.selectedCurrencyValue == "🇬🇧 GBP")
    }

    @Test
    func setNewCurrency() {
        let usdCurrencyStorage = MockCurrencyStorage()
        let viewModel = CurrencySceneViewModel(currencyStorage: usdCurrencyStorage, priceService: .mock())

        try? viewModel.setCurrency(.ars)

        #expect(usdCurrencyStorage.currency == Currency.ars.id)
        #expect(usdCurrencyStorage.currency == viewModel.currency.id)
    }
}
