// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemPreferencesRow
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

    private var state: GemPreferencesState {
        settings.preferences(currency: preferences.currency.toGem(), perpetualsEnabled: isPerpetualEnabled)
    }

    var title: String {
        Localized.Settings.Preferences.title
    }

    var leverageTitle: String {
        GemPreferencesRow.perpetualLeverage.title
    }

    var takeProfitTitle: String {
        GemPreferencesRow.perpetualTakeProfit.title
    }

    var stopLossTitle: String {
        GemPreferencesRow.perpetualStopLoss.title
    }

    var sections: [ListSection<PreferencesRowViewModel>] {
        state.sections.enumerated().map { index, section in
            ListSection(id: "\(index)", title: nil, image: nil, values: section.rows.map(rowViewModel))
        }
    }

    private func rowViewModel(_ row: GemPreferencesRow) -> PreferencesRowViewModel {
        PreferencesRowViewModel(id: String(describing: row), kind: kind(for: row), model: listItem(for: row))
    }

    private func kind(for row: GemPreferencesRow) -> PreferencesRowKind {
        switch row {
        case .currency: .currency
        case .language: .language
        case .appearance: .appearance
        case .networks: .networks
        case .contacts: .contacts
        case .perpetuals: .perpetuals
        case .perpetualLeverage: .perpetualLeverage
        case .perpetualTakeProfit: .perpetualTakeProfit
        case .perpetualStopLoss: .perpetualStopLoss
        }
    }

    private func listItem(for row: GemPreferencesRow) -> ListItemModel {
        switch row {
        case .currency: ListItemModel(title: row.title, subtitle: state.currency.text(), imageStyle: .settings(assetImage: row.assetImage))
        case .language: ListItemModel(title: row.title, subtitle: languageValue, imageStyle: .settings(assetImage: row.assetImage))
        case .appearance: ListItemModel(title: row.title, subtitle: appearanceValue, imageStyle: .settings(assetImage: row.assetImage))
        case .networks, .contacts, .perpetuals: ListItemModel(title: row.title, imageStyle: .settings(assetImage: row.assetImage))
        case .perpetualLeverage: ListItemModel(title: row.title, subtitle: defaultLeverageValue)
        case .perpetualTakeProfit: ListItemModel(title: row.title, subtitle: defaultTakeProfitValue)
        case .perpetualStopLoss: ListItemModel(title: row.title, subtitle: defaultStopLossValue)
        }
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
