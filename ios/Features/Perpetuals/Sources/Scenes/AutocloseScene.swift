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
        let viewState = model.viewState
        return List {
            if let positionRow = model.positionRow(viewState) {
                Section {
                    ListAssetItemView(row: positionRow)
                }
            }

            Section {
                ForEach(viewState.priceRows, id: \.self) { row in
                    GemListRowView(row: row)
                }
            }

            AutocloseInputSection(
                text: $model.takeProfitText,
                state: viewState.takeProfit,
                field: Field.takeProfit,
                focusedField: $focusedField,
            )

            AutocloseInputSection(
                text: $model.stopLossText,
                state: viewState.stopLoss,
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
                suggestions: model.percentSuggestions(viewState),
                onSelect: { model.onSelectPercent($0.value) },
                onDone: { focusedField = nil },
                button: StateButton(
                    text: Localized.Transfer.confirm,
                    type: model.confirmButtonType(viewState),
                    action: onSelectConfirm,
                ),
            )
        }
        .navigationTitle(model.title)
        .alertSheet($model.isPresentingAlertMessage)
        .navigationBarTitleDisplayMode(.inline)
        .toolbar { ToolbarDismissItem(type: .close, placement: .topBarLeading) }
        .onChange(of: focusedField, model.onChangeFocusField)
    }

    private func onSelectConfirm() {
        focusedField = nil
        model.onSelectConfirm()
    }
}
