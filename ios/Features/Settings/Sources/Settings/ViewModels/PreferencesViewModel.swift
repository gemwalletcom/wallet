// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemListRowTitle
import struct Gemstone.GemPreferencesInput
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
        let defaults = settings.perpetualDefaults()
        perpetualLeverage = LeverageOption(value: defaults.leverage)
        perpetualTakeProfit = AutocloseOption(value: defaults.takeProfitPercent)
        perpetualStopLoss = AutocloseOption(value: defaults.stopLossPercent)
    }

    var title: String {
        Localized.Settings.Preferences.title
    }

    var leverageTitle: String {
        GemListRowTitle.perpetualLeverage.text
    }

    var takeProfitTitle: String {
        GemListRowTitle.perpetualTakeProfit.text
    }

    var stopLossTitle: String {
        GemListRowTitle.perpetualStopLoss.text
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

    var takeProfitOptions: [AutocloseOption] {
        AutocloseOption.takeProfitOptions
    }

    var stopLossOptions: [AutocloseOption] {
        AutocloseOption.stopLossOptions
    }
}

// MARK: - ListSectionProvideable

extension PreferencesViewModel: ListSectionProvideable {
    public var sections: [ListSection<GemListSectionRow>] {
        settings.preferencesSections(
            input: GemPreferencesInput(
                currency: preferences.currency.toGem(),
                language: languageValue,
                appearance: appearanceValue,
                perpetualsEnabled: isPerpetualEnabled,
                perpetualLeverage: perpetualLeverage.displayText,
                perpetualTakeProfit: perpetualTakeProfit.displayText,
                perpetualStopLoss: perpetualStopLoss.displayText,
            ),
        ).listSections
    }
}

// MARK: - Actions

extension PreferencesViewModel {
    func onToggle(_ title: GemListRowTitle, _ isOn: Bool) {
        switch title {
        case .perpetuals: isPerpetualEnabled = isOn
        default: break
        }
    }

    func onSelect(_ title: GemListRowTitle) {
        switch title {
        case .perpetualLeverage: isPresentingLeveragePicker = true
        case .perpetualTakeProfit: isPresentingTakeProfitPicker = true
        case .perpetualStopLoss: isPresentingStopLossPicker = true
        default: break
        }
    }
}
