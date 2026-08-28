// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import protocol Gemstone.GemPriceServiceProtocol
import Components
import GemstoneServices
import Foundation
import protocol Gemstone.GemDeviceServiceProtocol
import Localization
import Primitives

@Observable
@MainActor
public final class CurrencySceneViewModel {
    private var currencyStorage: CurrencyStorable
    private let priceService: any GemPriceServiceProtocol
    private let deviceService: any GemDeviceServiceProtocol
    private let defaultCurrencies: [Currency] = [.usd, .eur, .gbp, .cny, .jpy, .inr, .rub]

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
        priceService: any GemPriceServiceProtocol,
        deviceService: any GemDeviceServiceProtocol,
    ) {
        self.currencyStorage = currencyStorage
        self.priceService = priceService
        self.deviceService = deviceService
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
        self.currency = currency
        try await priceService.changeCurrency(currency: currency.json())
    }

    func updateDevice() async {
        do {
            _ = try await deviceService.synchronize()
        } catch {
            debugLog("currency scene: device synchronize error \(error)")
        }
    }
}

// MARK: - Private

extension CurrencySceneViewModel {
    private var recommendedCurrencies: [Currency] {
        guard let current = Locale.current.currency,
              let currency = Currency(rawValue: current.identifier)
        else {
            return defaultCurrencies
        }
        return ([self.currency, currency] + defaultCurrencies).unique()
    }

    private var allCurrencies: [Currency] {
        Currency.nativeCurrencies.compactMap { Currency(rawValue: $0.identifier) }
    }
}
