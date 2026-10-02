// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import Style
import SwiftUI

struct NetworkFeeCustomScene: View {
    private enum Field: Hashable {
        case baseFee
        case rate
    }

    @State private var model: NetworkFeeCustomViewModel
    private let onConfirm: () -> Void
    @FocusState private var focusedField: Field?

    init(model: NetworkFeeCustomViewModel, onConfirm: @escaping () -> Void) {
        _model = State(initialValue: model)
        self.onConfirm = onConfirm
    }

    var body: some View {
        List {
            if let baseFeeField = model.baseFeeField {
                fieldSection(baseFeeField, text: $model.baseFeeInput, field: .baseFee)
            }
            fieldSection(model.rateField, text: $model.input, field: .rate)

            Section {
                ListItemView(model: model.networkFeeListItem)
            }
            .listSectionSpacing(.custom(.medium))
        }
        .contentMargins(.top, .scene.top, for: .scrollContent)
        .listSectionSpacing(.compact)
        .navigationTitle(model.title)
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .topBarTrailing) {
                Button("", systemImage: SystemImage.checkmark) {
                    model.confirm()
                    onConfirm()
                }
                .disabled(model.isConfirmEnabled == false)
            }
        }
        .onAppear {
            focusedField = .rate
        }
    }

    private func fieldSection(_ fieldModel: NetworkFeeCustomFieldModel, text: Binding<String>, field: Field) -> some View {
        Section {
            SuffixTextField(
                placeholder: fieldModel.placeholder,
                suffix: model.suffix,
                sanitizer: model.sanitize,
                alignment: .leading,
                text: text,
                field: field,
                focusedField: $focusedField,
            )
            .alignmentGuide(.listRowSeparatorLeading) { $0[.leading] }

            if let error = fieldModel.errorText {
                Text(error)
                    .textStyle(TextStyle(font: .footnote, color: Colors.red))
            }
        } header: {
            Text(fieldModel.title)
        } footer: {
            HStack {
                Text(fieldModel.hintTitle)
                Spacer()
                Text(fieldModel.hint ?? Placeholder.empty)
            }
            .font(.subheadline)
            .fontWeight(.semibold)
        }
    }
}
