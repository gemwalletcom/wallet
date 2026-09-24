// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import PrimitivesComponents
import Style
import SwiftUI

public struct AutocloseScene: View {
    public enum Field: Int, Hashable {
        case takeProfit
        case stopLoss
    }

    @FocusState private var focusedField: Field?
    @State private var model: AutocloseSceneViewModel

    public init(model: AutocloseSceneViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        List {
            if let positionItemViewModel = model.positionItemViewModel {
                Section {
                    ListAssetItemView(model: positionItemViewModel)
                }
            }

            Section {
                ForEach(model.priceRows, id: \.self) { row in
                    GemListRowView(row: row)
                }
            }

            AutocloseInputSection(
                inputModel: $model.input.takeProfit,
                sectionModel: model.takeProfitModel,
                field: Field.takeProfit,
                focusedField: $focusedField,
            )

            AutocloseInputSection(
                inputModel: $model.input.stopLoss,
                sectionModel: model.stopLossModel,
                field: Field.stopLoss,
                focusedField: $focusedField,
            )
        }
        .listSectionSpacing(.compact)
        .contentMargins(.top, .scene.top, for: .scrollContent)
        .scrollDismissesKeyboard(.interactively)
        .safeAreaView {
            InputAccessoryView(
                isEditing: model.isEditing(field: focusedField),
                suggestions: model.takeProfitModel.percentSuggestions,
                onSelect: { model.onSelectPercent($0.value) },
                onDone: { focusedField = nil },
                button: StateButton(
                    text: Localized.Transfer.confirm,
                    type: model.confirmButtonType,
                    action: onSelectConfirm,
                ),
            )
        }
        .navigationTitle(model.title)
        .navigationBarTitleDisplayMode(.inline)
        .toolbar { ToolbarDismissItem(type: .close, placement: .topBarLeading) }
        .onChange(of: focusedField, model.onChangeFocusField)
        .onChange(of: model.input.takeProfit.text) { _, _ in model.onChangePrice() }
        .onChange(of: model.input.stopLoss.text) { _, _ in model.onChangePrice() }
    }

    private func onSelectConfirm() {
        focusedField = nil
        model.onSelectConfirm()
    }
}
