// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemListRow
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
        ListSectionView(sections: model.sections) { row in
            content(for: row)
                .listRowInsets(.assetListRowInsets)
        }
        .contentMargins(.top, .scene.top, for: .scrollContent)
        .listSectionSpacing(.compact)
        .navigationTitle(model.title)
        .sheet(isPresented: $model.isPresentingLeveragePicker) {
            WheelPickerSheet(
                title: model.leverageTitle,
                options: model.leverageOptions,
                selection: $model.perpetualLeverage,
            )
        }
        .sheet(isPresented: $model.isPresentingTakeProfitPicker) {
            WheelPickerSheet(
                title: model.takeProfitTitle,
                options: model.takeProfitOptions,
                selection: $model.perpetualTakeProfit,
            )
        }
        .sheet(isPresented: $model.isPresentingStopLossPicker) {
            WheelPickerSheet(
                title: model.stopLossTitle,
                options: model.stopLossOptions,
                selection: $model.perpetualStopLoss,
            )
        }
    }

    @ViewBuilder
    private func content(for row: GemListRow) -> some View {
        switch PreferencesRowDestination(row: row) {
        case .currency:
            link(row, to: Scenes.Currency())
        case .language:
            NavigationCustomLink(with: GemListRowView(row: row), action: onSelectLanguage)
        case .appearance:
            link(row, to: Scenes.Appearance())
        case .networks:
            link(row, to: Scenes.Chains())
        case .contacts:
            link(row, to: Scenes.Contacts())
        case .none:
            if case .picker = row {
                GemListRowView(row: row, onSelect: model.onSelect)
                    .padding(.leading, Sizing.image.asset - .tiny)
            } else {
                GemListRowView(row: row, onToggle: model.onToggle)
            }
        }
    }

    private func link(_ row: GemListRow, to scene: some Hashable) -> some View {
        NavigationLink(value: scene) {
            GemListRowView(row: row)
        }
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
