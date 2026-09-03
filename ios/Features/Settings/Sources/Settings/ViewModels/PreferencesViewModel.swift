// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import protocol Gemstone.GemPreferencesServiceProtocol
import GemstonePrimitives
import Localization
import Preferences
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

@Observable
@MainActor
public final class PreferencesViewModel {
    private let preferences: ObservablePreferences
    private let service: any GemPreferencesServiceProtocol
    private let currencyModel: CurrencySceneViewModel

    var isPresentingLeveragePicker = false
    var isPresentingTakeProfitPicker = false
    var isPresentingStopLossPicker = false

    public init(
        currencyModel: CurrencySceneViewModel,
        service: any GemPreferencesServiceProtocol,
        preferences: ObservablePreferences,
    ) {
        self.currencyModel = currencyModel
        self.service = service
        self.preferences = preferences
        perpetualLeverage = LeverageOption(value: service.getPerpetualLeverage())
        perpetualTakeProfit = AutocloseOption(value: service.getPerpetualTakeProfitPercent())
        perpetualStopLoss = AutocloseOption(value: service.getPerpetualStopLossPercent())
    }

    var title: String {
        Localized.Settings.Preferences.title
    }

    var currencyTitle: String {
        Localized.Settings.currency
    }

    var currencyValue: String {
        currencyModel.selectedCurrencyValue
    }

    var currencyImage: AssetImage {
        AssetImage.image(Images.Settings.currency)
    }

    var languageTitle: String {
        Localized.Settings.language
    }

    var languageValue: String {
        guard let code = Locale.current.language.languageCode?.identifier else {
            return ""
        }
        return Locale.current.localizedString(forLanguageCode: code)?.capitalized ?? ""
    }

    var languageImage: AssetImage {
        AssetImage.image(Images.Settings.language)
    }

    var networksTitle: String {
        Localized.Settings.Networks.title
    }

    var networksImage: AssetImage {
        AssetImage.image(Images.Settings.networks)
    }

    var contactsTitle: String {
        Localized.Contacts.title
    }

    var contactsImage: AssetImage {
        AssetImage.image(Images.Settings.contacts)
    }

    var appearanceTitle: String {
        Localized.Settings.appearanceTitle
    }

    var appearanceImage: AssetImage {
        AssetImage.image(Images.Settings.appearance)
    }

    var appearanceValue: String {
        preferences.appearance.title
    }

    var isPerpetualEnabled: Bool {
        get { preferences.isPerpetualEnabled }
        set { preferences.isPerpetualEnabled = newValue }
    }

    var perpetualsTitle: String {
        Localized.Perpetuals.title
    }

    var perpetualsImage: AssetImage {
        AssetImage.image(Images.Settings.perpetuals)
    }

    var perpetualLeverage: LeverageOption {
        didSet { persist { try service.setPerpetualLeverage(leverage: perpetualLeverage.value) } }
    }

    var defaultLeverageTitle: String {
        Localized.Settings.Preferences.Perpetual.defaultLeverage
    }

    var defaultLeverageValue: String {
        "\(perpetualLeverage.value)x"
    }

    var leverageOptions: [LeverageOption] {
        LeverageOption.allOptions
    }

    var perpetualTakeProfit: AutocloseOption {
        didSet { persist { try service.setPerpetualTakeProfitPercent(percent: perpetualTakeProfit.value) } }
    }

    var perpetualStopLoss: AutocloseOption {
        didSet { persist { try service.setPerpetualStopLossPercent(percent: perpetualStopLoss.value) } }
    }

    private func persist(_ write: () throws -> Void) {
        do {
            try write()
        } catch {
            debugLog("preferences write error: \(error)")
        }
    }

    var defaultTakeProfitTitle: String {
        Localized.Settings.Preferences.Perpetual.defaultTakeProfit
    }

    var defaultStopLossTitle: String {
        Localized.Settings.Preferences.Perpetual.defaultStopLoss
    }

    var defaultTakeProfitValue: String {
        perpetualTakeProfit.displayText
    }

    var defaultStopLossValue: String {
        perpetualStopLoss.displayText
    }

    var takeProfitOptions: [AutocloseOption] {
        AutocloseOption.takeProfitOptions
    }

    var stopLossOptions: [AutocloseOption] {
        AutocloseOption.stopLossOptions
    }
}

// MARK: - Actions

extension PreferencesViewModel {
    func onSelectLeverage() {
        isPresentingLeveragePicker = true
    }

    func onSelectTakeProfit() {
        isPresentingTakeProfitPicker = true
    }

    func onSelectStopLoss() {
        isPresentingStopLossPicker = true
    }
}
