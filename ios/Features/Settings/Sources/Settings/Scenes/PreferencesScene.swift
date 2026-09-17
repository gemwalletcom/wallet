// Copyright (c). Gem Wallet. All rights reserved.

import Components
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
        List {
            Group {
                ForEach(model.sections) { section in
                    Section {
                        ForEach(section.values) { row in
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
    private func content(for row: PreferencesRowViewModel) -> some View {
        switch row.kind {
        case .currency:
            NavigationLink(value: Scenes.Currency()) {
                ListItemView(model: row.model)
            }
        case .language:
            NavigationCustomLink(
                with: ListItemView(model: row.model),
                action: onSelectLanguage,
            )
        case .appearance:
            NavigationLink(value: Scenes.Appearance()) {
                ListItemView(model: row.model)
            }
        case .networks:
            NavigationLink(value: Scenes.Chains()) {
                ListItemView(model: row.model)
            }
        case .contacts:
            NavigationLink(value: Scenes.Contacts()) {
                ListItemView(model: row.model)
            }
        case .perpetuals:
            ListItemToggleView(
                isOn: $model.isPerpetualEnabled,
                title: row.model.title ?? .empty,
                imageStyle: row.model.imageStyle,
            )
        case .perpetualLeverage:
            perpetualLink(row, action: model.onSelectLeverage)
        case .perpetualTakeProfit:
            perpetualLink(row, action: model.onSelectTakeProfit)
        case .perpetualStopLoss:
            perpetualLink(row, action: model.onSelectStopLoss)
        }
    }

    private func perpetualLink(_ row: PreferencesRowViewModel, action: @escaping @MainActor () -> Void) -> some View {
        NavigationCustomLink(with: ListItemView(model: row.model), action: action)
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
