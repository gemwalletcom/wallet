// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemPreferencesState
import protocol Gemstone.GemSettingsServiceProtocol
import Foundation
import struct Gemstone.GemPerpetualDefaults
import GemstonePrimitives
import Localization
import GemstoneServices
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

@Observable
@MainActor
public final class PreferencesViewModel {
    private let preferences: ObservablePreferences
    private let settings: any GemSettingsServiceProtocol

    var isPresentingLeveragePicker = false
    var isPresentingTakeProfitPicker = false
    var isPresentingStopLossPicker = false

    public init(
        settings: any GemSettingsServiceProtocol,
        preferences: ObservablePreferences,
    ) {
        self.settings = settings
        self.preferences = preferences
        let defaults = settings.preferences(currency: preferences.currency.toGem(), perpetualsEnabled: preferences.isPerpetualEnabled).perpetualDefaults
        perpetualLeverage = LeverageOption(value: defaults.leverage)
        perpetualTakeProfit = AutocloseOption(value: defaults.takeProfitPercent)
        perpetualStopLoss = AutocloseOption(value: defaults.stopLossPercent)
    }

    var state: GemPreferencesState {
        settings.preferences(currency: preferences.currency.toGem(), perpetualsEnabled: isPerpetualEnabled)
    }

    var title: String {
        Localized.Settings.Preferences.title
    }

    var languageValue: String {
        guard let code = Locale.current.language.languageCode?.identifier else {
            return ""
        }
        return Locale.current.localizedString(forLanguageCode: code)?.capitalized ?? ""
    }

    var appearanceValue: String {
        preferences.appearance.title
    }

    var isPerpetualEnabled: Bool {
        get { preferences.isPerpetualEnabled }
        set { preferences.isPerpetualEnabled = newValue }
    }

    var perpetualLeverage: LeverageOption {
        didSet { persistPerpetualDefaults() }
    }

    var defaultLeverageValue: String {
        "\(perpetualLeverage.value)x"
    }

    var leverageOptions: [LeverageOption] {
        LeverageOption.allOptions
    }

    var perpetualTakeProfit: AutocloseOption {
        didSet { persistPerpetualDefaults() }
    }

    var perpetualStopLoss: AutocloseOption {
        didSet { persistPerpetualDefaults() }
    }

    private func persistPerpetualDefaults() {
        do {
            try settings.setPerpetualDefaults(
                defaults: GemPerpetualDefaults(
                    leverage: perpetualLeverage.value,
                    takeProfitPercent: perpetualTakeProfit.value,
                    stopLossPercent: perpetualStopLoss.value,
                ),
            )
        } catch {
            debugLog("preferences write error: \(error)")
        }
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
