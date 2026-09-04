// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
import Foundation
import GemstoneServices
import Primitives
@testable import Settings
import Testing

private final class MockCurrencyStorage: CurrencyStorable, @unchecked Sendable {
    var currency: Currency
    init(currency: Currency = .usd) {
        self.currency = currency
    }
}

@MainActor
struct CurrencySceneViewModelTests {
    private var storage = MockCurrencyStorage()

    @Test
    func uSDCurrencyValue() {
        let usdCurrencyStorage = MockCurrencyStorage()
        let viewModel = CurrencySceneViewModel(currencyStorage: usdCurrencyStorage, service: GemCurrencyServiceMock())

        #expect(viewModel.selectedCurrencyValue == "🇺🇸 USD")
    }

    @Test
    func gBPCurrencyValue() {
        let gbpCurrencyStorage = MockCurrencyStorage(currency: .gbp)
        let viewModel = CurrencySceneViewModel(currencyStorage: gbpCurrencyStorage, service: GemCurrencyServiceMock())
        #expect(viewModel.selectedCurrencyValue == "🇬🇧 GBP")
    }

    @Test
    func setNewCurrency() async throws {
        let usdCurrencyStorage = MockCurrencyStorage()
        let service = GemCurrencyServiceMock()
        let viewModel = CurrencySceneViewModel(currencyStorage: usdCurrencyStorage, service: service)

        try await viewModel.setCurrency(.ars)

        #expect(service.setCurrencies == [Currency.ars.rawValue])
        #expect(usdCurrencyStorage.currency == .ars)
        #expect(usdCurrencyStorage.currency == viewModel.currency)
    }

    @Test
    func aFailedChangeLeavesTheStoredCurrency() async {
        let usdCurrencyStorage = MockCurrencyStorage()
        let viewModel = CurrencySceneViewModel(currencyStorage: usdCurrencyStorage, service: GemCurrencyServiceMock(error: AnyError("offline")))

        await #expect(throws: (any Error).self) { try await viewModel.setCurrency(.ars) }
        #expect(usdCurrencyStorage.currency == .usd)
    }
}
