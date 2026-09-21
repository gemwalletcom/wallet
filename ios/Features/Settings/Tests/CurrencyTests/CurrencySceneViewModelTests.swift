// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemCurrencyRow
import struct Gemstone.GemCurrencySection
import class Gemstone.GemPreferencesService
import GemstonePrimitives
import GemstonePrimitivesTestKit
import GemstoneServices
import Primitives
@testable import Settings
import Testing

@MainActor
struct CurrencySceneViewModelTests {
    @Test
    func searchUsesGemstoneSections() {
        let preferences = GemPreferencesService(store: GemPreferencesStoreMock())
        let service = GemCurrencyServiceMock(preferencesService: preferences)
        let viewModel = CurrencySceneViewModel(preferences: ObservablePreferences(preferencesService: preferences), service: service)
        service.sectionsValue = [
            GemCurrencySection(kind: .all, rows: [GemCurrencyRow(currency: Currency.ars.toGem(), title: "ARS", isSelected: false)]),
        ]

        viewModel.searchQuery = " arS "
        #expect(viewModel.sections == service.sectionsValue)
        viewModel.searchQuery = ""
        #expect(viewModel.sections == service.sectionsValue)
        #expect(service.queries == [" arS ", ""])
    }

    @Test
    func setNewCurrency() async throws {
        let preferences = GemPreferencesService(store: GemPreferencesStoreMock())
        let observablePreferences = ObservablePreferences(preferencesService: preferences)
        let service = GemCurrencyServiceMock(preferencesService: preferences)
        let viewModel = CurrencySceneViewModel(preferences: observablePreferences, service: service)

        try await viewModel.setCurrency(Currency.ars.toGem())

        #expect(service.setCurrencies == [Currency.ars.toGem()])
        #expect(observablePreferences.currency == .ars)
    }

    @Test
    func aFailedChangeLeavesTheStoredCurrency() async {
        let preferences = GemPreferencesService(store: GemPreferencesStoreMock())
        let observablePreferences = ObservablePreferences(preferencesService: preferences)
        let viewModel = CurrencySceneViewModel(
            preferences: observablePreferences,
            service: GemCurrencyServiceMock(preferencesService: preferences, error: AnyError("offline")),
        )

        await #expect(throws: (any Error).self) {
            try await viewModel.setCurrency(Currency.ars.toGem())
        }
        #expect(observablePreferences.currency == .usd)
    }
}
