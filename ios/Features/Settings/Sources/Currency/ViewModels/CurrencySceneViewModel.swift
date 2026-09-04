// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
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
        get {
            guard let currency = Currency(rawValue: currencyStorage.currency) else {
                fatalError("unsupported currency")
            }
            return currency
        }
        set {
            currencyStorage.currency = newValue.rawValue
        }
    }

    public init(
        currencyStorage: CurrencyStorable,
        service: any GemCurrencyServiceProtocol,
    ) {
        self.currencyStorage = currencyStorage
        self.service = service
    }

    public var selectedCurrencyValue: String {
        let model = CurrencyViewModel(currency: currency)
        if let flag = model.flag {
            return "\(flag) \(currency.rawValue)"
        }
        return currency.rawValue
    }

    var title: String {
        Localized.Settings.currency
    }

    var list: [ListItemValueSection<CurrencyViewModel>] {
        let recommendedVMs = recommendedCurrencies.map { CurrencyViewModel(currency: $0) }
        let recommendedValues = recommendedVMs.map { ListItemValue(title: $0.title, value: $0) }
        let allVMs = allCurrencies.map { CurrencyViewModel(currency: $0) }
        let allValues = allVMs.map { ListItemValue(title: $0.title, value: $0) }

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

    private var recommendedCurrencies: [Currency] {
        service.recommendedCurrencies(locale: localeCurrency?.rawValue).compactMap { Currency(rawValue: $0) }
    }

    private var allCurrencies: [Currency] {
        service.otherCurrencies(locale: localeCurrency?.rawValue).compactMap { Currency(rawValue: $0) }
    }
}
