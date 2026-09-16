// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import GemstonePrimitivesTestKit
import Foundation
import GemstoneServices
import Primitives
@testable import Settings
import Testing

private final class MockCurrencyStorage: CurrencyStorable, @unchecked Sendable {
    var currency: Currency = .usd
}

@MainActor
struct CurrencySceneViewModelTests {
    @Test
    func setNewCurrency() async throws {
        let usdCurrencyStorage = MockCurrencyStorage()
        let service = GemCurrencyServiceMock()
        let viewModel = CurrencySceneViewModel(currencyStorage: usdCurrencyStorage, service: service)

        try await viewModel.setCurrency(.ars)

        #expect(service.setCurrencies == [Currency.ars.toGem()])
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
