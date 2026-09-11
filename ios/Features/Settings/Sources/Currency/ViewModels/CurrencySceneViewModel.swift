// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import struct Gemstone.GemCurrencies
import protocol Gemstone.GemCurrencyServiceProtocol
import Components
import GemstoneServices
import Foundation
import Localization
import Primitives

@Observable
@MainActor
public final class CurrencySceneViewModel {
    private var currencyStorage: CurrencyStorable
    private let service: any GemCurrencyServiceProtocol

    private(set) var currency: Currency {
        get { currencyStorage.currency }
        set { currencyStorage.currency = newValue }
    }

    public init(
        currencyStorage: CurrencyStorable,
        service: any GemCurrencyServiceProtocol,
    ) {
        self.currencyStorage = currencyStorage
        self.service = service
    }

    public var selectedCurrencyValue: String {
        "\(currencies.selected.flag) \(currency.rawValue)"
    }

    var title: String {
        Localized.Settings.currency
    }

    var list: [ListItemValueSection<CurrencyViewModel>] {
        let currencies = currencies
        let recommendedValues = currencies.recommended.map(CurrencyViewModel.init).map { ListItemValue(title: $0.title, value: $0) }
        let allValues = currencies.other.map(CurrencyViewModel.init).map { ListItemValue(title: $0.title, value: $0) }

        return [
            ListItemValueSection(
                section: Localized.Common.recommended,
                values: recommendedValues,
            ),
            ListItemValueSection(
                section: Localized.Common.all,
                values: allValues,
            ),
        ]
    }

    func setCurrency(_ currency: Currency) async throws {
        try await service.setCurrency(currency: currency.rawValue)
        self.currency = currency
    }
}

// MARK: - Private

extension CurrencySceneViewModel {
    private var localeCurrency: Currency? {
        Locale.current.currency.flatMap { Currency(rawValue: $0.identifier) }
    }

    private var currencies: GemCurrencies {
        service.currencies(locale: localeCurrency?.rawValue)
    }
}
