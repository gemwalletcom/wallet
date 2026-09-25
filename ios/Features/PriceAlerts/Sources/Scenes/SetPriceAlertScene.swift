// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemPriceAlertViewState
import Localization
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI

struct SetPriceAlertScene: View {
    @State private var model: SetPriceAlertViewModel

    @FocusState private var focusedField: Bool

    init(model: SetPriceAlertViewModel) {
        _model = State(initialValue: model)
    }

    var body: some View {
        let viewState = model.viewState
        return List {
            Section {
                VStack(spacing: .small) {
                    Text(model.directionTitle(viewState))
                        .textStyle(.subHeadline)

                    CurrencyInputView(
                        text: $model.state.amount,
                        config: model.currencyInputConfig(viewState),
                    )
                    .focused($focusedField)
                }
            }
            .cleanListRow()

            Section {
                ListAssetItemView(row: model.assetRow)
            }
        }
        .bindQuery(model.assetQuery)
        .safeAreaView {
            inputAccessoryView(viewState)
        }
        .toolbar {
            ToolbarItem(placement: .principal) {
                alertTypePickerView
            }
        }
        .onChange(of: model.state.type, model.onChangeAlertType)
        .alertSheet($model.isPresentingAlertMessage)
        .onAppear {
            focusedField = true
        }
    }
}

// MARK: - UI

extension SetPriceAlertScene {
    var alertTypePickerView: some View {
        Picker("", selection: $model.state.type) {
            Text(Localized.Asset.price)
                .tag(SetPriceAlertType.price)
            Text(Localized.Common.percentage)
                .tag(SetPriceAlertType.percentage)
        }
        .pickerStyle(.segmented)
        .fixedSize()
    }

    private func inputAccessoryView(_ viewState: GemPriceAlertViewState) -> some View {
        InputAccessoryView(
            isEditing: focusedField && model.state.amount.isEmpty,
            suggestions: model.suggestions(viewState),
            onSelect: onSelectSuggestion,
            onDone: { focusedField = false },
            button: StateButton(
                text: Localized.Transfer.confirm,
                type: .primary(model.confirmButtonState(viewState)),
                action: confirm,
            ),
        )
    }
}

// MARK: - Actions

extension SetPriceAlertScene {
    func onSelectSuggestion(_ suggestion: some SuggestionViewable) {
        model.onSelectSuggestion(suggestion)
    }

    func confirm() {
        Task {
            await model.setPriceAlert()
        }
    }
}
