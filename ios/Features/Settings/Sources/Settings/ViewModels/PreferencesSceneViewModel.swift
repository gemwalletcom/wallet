// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemGroup
import enum Gemstone.GemListRowTitle
import struct Gemstone.GemPerpetualDefaults
import struct Gemstone.GemPerpetualPickers
import struct Gemstone.GemPickerOption
import enum Gemstone.GemRowAction
import protocol Gemstone.GemSettingsServiceProtocol
import GemstonePrimitives
import GemstoneServices
import Localization
import Primitives
import PrimitivesComponents
import SwiftUI

@Observable
@MainActor
public final class PreferencesSceneViewModel {
    private let preferences: ObservablePreferences
    private let settings: any GemSettingsServiceProtocol
    private let pickers: GemPerpetualPickers

    var isPresentingSheet: PreferencesSheetType?

    public init(
        settings: any GemSettingsServiceProtocol,
        preferences: ObservablePreferences,
    ) {
        self.settings = settings
        self.preferences = preferences
        let defaults = settings.perpetualDefaults()
        pickers = settings.perpetualPickers()
        perpetualLeverage = Self.option(pickers.leverage, defaults.leverage)
        perpetualMarginType = Self.option(pickers.marginType, defaults.marginType)
        perpetualTakeProfit = Self.option(pickers.takeProfit, defaults.takeProfitPercent)
        perpetualStopLoss = Self.option(pickers.stopLoss, defaults.stopLossPercent)
    }

    var title: String {
        Localized.Settings.Preferences.title
    }

    var leverageTitle: String {
        GemListRowTitle.perpetualLeverage.text
    }

    var marginTypeTitle: String {
        GemListRowTitle.perpetualMarginType.text
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

    var perpetualLeverage: GemPickerOption {
        didSet { persistPerpetualDefaults() }
    }

    var leverageOptions: [GemPickerOption] {
        pickers.leverage
    }

    var perpetualMarginType: GemPickerOption {
        didSet { persistPerpetualDefaults() }
    }

    var marginTypeOptions: [GemPickerOption] {
        pickers.marginType
    }

    var perpetualTakeProfit: GemPickerOption {
        didSet { persistPerpetualDefaults() }
    }

    var perpetualStopLoss: GemPickerOption {
        didSet { persistPerpetualDefaults() }
    }

    private func persistPerpetualDefaults() {
        do {
            try settings.setPerpetualDefaults(
                defaults: GemPerpetualDefaults(
                    leverage: perpetualLeverage.value,
                    marginType: perpetualMarginType.value,
                    takeProfitPercent: perpetualTakeProfit.value,
                    stopLossPercent: perpetualStopLoss.value,
                ),
            )
        } catch {
            debugLog("preferences write error: \(error)")
        }
    }

    var takeProfitOptions: [GemPickerOption] {
        pickers.takeProfit
    }

    var stopLossOptions: [GemPickerOption] {
        pickers.stopLoss
    }

    private static func option(_ options: [GemPickerOption], _ value: UInt8) -> GemPickerOption {
        options.first { $0.value == value } ?? GemPickerOption(value: value, label: .none)
    }
}

public extension PreferencesSceneViewModel {
    var sections: [GemGroup] {
        preferences.changes
        return settings.preferencesSections(language: languageValue, appearance: appearanceValue)
    }
}

// MARK: - Actions

extension PreferencesSceneViewModel {
    func onToggle(_ action: GemRowAction, _ isOn: Bool) {
        switch action {
        case .perpetuals: isPerpetualEnabled = isOn
        default: break
        }
    }

    func onSelect(_ action: GemRowAction) {
        switch action {
        case .perpetualLeverage: isPresentingSheet = .leverage
        case .perpetualMarginType: isPresentingSheet = .marginType
        case .perpetualTakeProfit: isPresentingSheet = .takeProfit
        case .perpetualStopLoss: isPresentingSheet = .stopLoss
        default: break
        }
    }
}
