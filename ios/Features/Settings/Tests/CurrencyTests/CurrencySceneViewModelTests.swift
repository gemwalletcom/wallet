// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import class Gemstone.GemPriceService
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
import Foundation
import GemstoneServices
import Primitives
@testable import Settings
import Testing

private final class MockCurrencyStorage: CurrencyStorable, @unchecked Sendable {
    var currency: String
    init(currency: String = "USD") {
        self.currency = currency
    }
}

@MainActor
struct CurrencySceneViewModelTests {
    private var storage = MockCurrencyStorage()

    @Test
    func uSDCurrencyValue() {
        let usdCurrencyStorage = MockCurrencyStorage()
        let viewModel = CurrencySceneViewModel(currencyStorage: usdCurrencyStorage, priceService: GemPriceService.mock(), deviceService: GemDeviceServiceMock())

        #expect(viewModel.selectedCurrencyValue == "🇺🇸 USD")
    }

    @Test
    func gBPCurrencyValue() {
        let gbpCurrencyStorage = MockCurrencyStorage(currency: "GBP")
        let viewModel = CurrencySceneViewModel(currencyStorage: gbpCurrencyStorage, priceService: GemPriceService.mock(), deviceService: GemDeviceServiceMock())
        #expect(viewModel.selectedCurrencyValue == "🇬🇧 GBP")
    }

    @Test
    func setNewCurrency() async throws {
        let priceService = GemPriceService.mock()
        try await priceService.updateRates(rates: [FiatRate(symbol: .ars, rate: 1200).json()], currency: Currency.ars.rawValue)
        let usdCurrencyStorage = MockCurrencyStorage()
        let deviceService = GemDeviceServiceMock()
        let viewModel = CurrencySceneViewModel(currencyStorage: usdCurrencyStorage, priceService: priceService, deviceService: deviceService)

        try await viewModel.setCurrency(.ars)
        await viewModel.updateDevice()

        #expect(usdCurrencyStorage.currency == Currency.ars.id)
        #expect(usdCurrencyStorage.currency == viewModel.currency.id)
        #expect(await deviceService.synchronizeIfNeededCalls == 1)
    }
}
