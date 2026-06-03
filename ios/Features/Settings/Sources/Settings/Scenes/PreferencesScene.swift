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
                Section {
                    NavigationLink(value: Scenes.Currency()) {
                        ListItemView(
                            title: model.currencyTitle,
                            subtitle: model.currencyValue,
                            imageStyle: .settings(assetImage: model.currencyImage),
                        )
                    }

                    NavigationCustomLink(
                        with: ListItemView(
                            title: model.languageTitle,
                            subtitle: model.languageValue,
                            imageStyle: .settings(assetImage: model.languageImage),
                        ),
                        action: onSelectLanguage,
                    )

                    NavigationLink(value: Scenes.Chains()) {
                        ListItemView(
                            title: model.networksTitle,
                            imageStyle: .settings(assetImage: model.networksImage),
                        )
                    }

                    NavigationLink(value: Scenes.Contacts()) {
                        ListItemView(
                            title: model.contactsTitle,
                            imageStyle: .settings(assetImage: model.contactsImage),
                        )
                    }
                }
                Section {
                    ListItemToggleView(
                        isOn: $model.isPerpetualEnabled,
                        title: model.perpetualsTitle,
                        imageStyle: .settings(assetImage: model.perpetualsImage),
                    )

                    if model.isPerpetualEnabled {
                        perpetualLink(
                            title: model.defaultLeverageTitle,
                            value: model.defaultLeverageValue,
                            action: model.onSelectLeverage,
                        )
                        perpetualLink(
                            title: model.defaultTakeProfitTitle,
                            value: model.defaultTakeProfitValue,
                            action: model.onSelectTakeProfit,
                        )
                        perpetualLink(
                            title: model.defaultStopLossTitle,
                            value: model.defaultStopLossValue,
                            action: model.onSelectStopLoss,
                        )
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
                title: model.defaultLeverageTitle,
                options: model.leverageOptions,
                selection: $model.perpetualLeverage,
            )
        }
        .sheet(isPresented: $model.isPresentingTakeProfitPicker) {
            WheelPickerSheet(
                title: model.defaultTakeProfitTitle,
                options: model.takeProfitOptions,
                selection: $model.perpetualTakeProfit,
            )
        }
        .sheet(isPresented: $model.isPresentingStopLossPicker) {
            WheelPickerSheet(
                title: model.defaultStopLossTitle,
                options: model.stopLossOptions,
                selection: $model.perpetualStopLoss,
            )
        }
    }

    private func perpetualLink(
        title: String,
        value: String,
        action: @escaping @MainActor () -> Void,
    ) -> some View {
        NavigationCustomLink(
            with: ListItemView(title: title, subtitle: value),
            action: action,
        )
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
