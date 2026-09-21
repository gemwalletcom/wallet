// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.Currency
import struct Gemstone.GemCurrencySection
import protocol Gemstone.GemCurrencyServiceProtocol
import GemstonePrimitives
import GemstoneServices
import Localization
import Primitives

@Observable
@MainActor
public final class CurrencySceneViewModel {
    private let preferences: ObservablePreferences
    private let service: any GemCurrencyServiceProtocol

    var isPresentingAlertMessage: AlertMessage?
    var searchQuery = ""

    public init(preferences: ObservablePreferences, service: any GemCurrencyServiceProtocol) {
        self.preferences = preferences
        self.service = service
    }

    var title: String {
        Localized.Settings.currency
    }

    var sections: [GemCurrencySection] {
        service.sections(
            currency: preferences.currency.toGem(),
            locale: Locale.current.currency.flatMap { Primitives.Currency(rawValue: $0.identifier) }?.toGem(),
            query: searchQuery,
            localizedNames: Dictionary(uniqueKeysWithValues: Primitives.Currency.allCases.map {
                ($0.rawValue, Locale.current.localizedString(forCurrencyCode: $0.rawValue) ?? .empty)
            }),
        )
    }
}

extension CurrencySceneViewModel {
    func setCurrency(_ currency: Gemstone.Currency) async throws {
        try await service.setCurrency(currency: currency)
        preferences.reload()
    }
}
