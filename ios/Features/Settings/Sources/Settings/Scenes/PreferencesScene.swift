// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemPreferencesRow
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct PreferencesScene: View {
    @Environment(\.openURL) private var openURL

    @State private var model: PreferencesViewModel

    public init(model: PreferencesViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        let state = model.state
        List {
            Group {
                ForEach(Array(state.sections.enumerated()), id: \.offset) { _, section in
                    Section {
                        ForEach(section.rows, id: \.self) { row in
                            content(for: row)
                        }
                    }
                }
            }
            .listRowInsets(.assetListRowInsets)
        }
        .contentMargins(.top, .scene.top, for: .scrollContent)
        .listSectionSpacing(.compact)
        .navigationTitle(model.title)
        .sheet(isPresented: $model.isPresentingLeveragePicker) {
            WheelPickerSheet(
                title: GemPreferencesRow.perpetualLeverage.title,
                options: model.leverageOptions,
                selection: $model.perpetualLeverage,
            )
        }
        .sheet(isPresented: $model.isPresentingTakeProfitPicker) {
            WheelPickerSheet(
                title: GemPreferencesRow.perpetualTakeProfit.title,
                options: model.takeProfitOptions,
                selection: $model.perpetualTakeProfit,
            )
        }
        .sheet(isPresented: $model.isPresentingStopLossPicker) {
            WheelPickerSheet(
                title: GemPreferencesRow.perpetualStopLoss.title,
                options: model.stopLossOptions,
                selection: $model.perpetualStopLoss,
            )
        }
    }

    @ViewBuilder
    private func content(for row: GemPreferencesRow) -> some View {
        switch row {
        case .currency:
            NavigationLink(value: Scenes.Currency()) {
                ListItemView(model: model.listItem(for: row))
            }
        case .language:
            NavigationCustomLink(
                with: ListItemView(model: model.listItem(for: row)),
                action: onSelectLanguage,
            )
        case .appearance:
            NavigationLink(value: Scenes.Appearance()) {
                ListItemView(model: model.listItem(for: row))
            }
        case .networks:
            NavigationLink(value: Scenes.Chains()) {
                ListItemView(model: model.listItem(for: row))
            }
        case .contacts:
            NavigationLink(value: Scenes.Contacts()) {
                ListItemView(model: model.listItem(for: row))
            }
        case .perpetuals:
            ListItemToggleView(
                isOn: $model.isPerpetualEnabled,
                title: row.title,
                imageStyle: .settings(assetImage: row.assetImage),
            )
        case .perpetualLeverage:
            perpetualLink(row, action: model.onSelectLeverage)
        case .perpetualTakeProfit:
            perpetualLink(row, action: model.onSelectTakeProfit)
        case .perpetualStopLoss:
            perpetualLink(row, action: model.onSelectStopLoss)
        }
    }

    private func perpetualLink(_ row: GemPreferencesRow, action: @escaping @MainActor () -> Void) -> some View {
        NavigationCustomLink(with: ListItemView(model: model.listItem(for: row)), action: action)
            .padding(.leading, Sizing.image.asset - .tiny)
    }
}

// MARK: - Actions

extension PreferencesScene {
    private func onSelectLanguage() {
        if let settingsURL = URL(string: UIApplication.openSettingsURLString) {
            openURL(settingsURL)
        }
    }
}
