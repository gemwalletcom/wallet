// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
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
    private let currencyModel: CurrencySceneViewModel

    var isPresentingLeveragePicker = false
    var isPresentingTakeProfitPicker = false
    var isPresentingStopLossPicker = false

    public init(
        currencyModel: CurrencySceneViewModel,
        preferences: ObservablePreferences = .default,
    ) {
        self.currencyModel = currencyModel
        self.preferences = preferences
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

    private var leverage: UInt8 {
        preferences.perpetualLeverage == 0 ? PerpetualConfig.defaultLeverage : preferences.perpetualLeverage
    }

    var perpetualLeverage: LeverageOption {
        get { LeverageOption(value: leverage) }
        set { preferences.perpetualLeverage = newValue.value }
    }

    var defaultLeverageTitle: String {
        Localized.Settings.Preferences.defaultLeverage
    }

    var defaultLeverageValue: String {
        "\(leverage)x"
    }

    var leverageOptions: [LeverageOption] {
        LeverageOption.allOptions
    }

    var perpetualTakeProfit: AutocloseOption {
        get { AutocloseOption(value: preferences.perpetualTakeProfit) }
        set { preferences.perpetualTakeProfit = newValue.value }
    }

    var perpetualStopLoss: AutocloseOption {
        get { AutocloseOption(value: preferences.perpetualStopLoss) }
        set { preferences.perpetualStopLoss = newValue.value }
    }

    var defaultTakeProfitTitle: String {
        Localized.Settings.Preferences.defaultTakeProfit
    }

    var defaultStopLossTitle: String {
        Localized.Settings.Preferences.defaultStopLoss
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
