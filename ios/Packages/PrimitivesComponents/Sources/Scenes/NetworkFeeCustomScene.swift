// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Style
import SwiftUI

public struct NetworkFeeCustomScene: View {
    private enum Field: Hashable {
        case input
    }

    @State private var model: NetworkFeeCustomViewModel
    @FocusState private var focusedField: Field?

    public init(model: NetworkFeeCustomViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        List {
            Section {
                HStack(spacing: .small) {
                    Text(model.title)
                    SuffixTextField(
                        placeholder: model.placeholder,
                        suffix: model.suffix,
                        sanitizer: model.sanitize,
                        text: $model.input,
                        field: Field.input,
                        focusedField: $focusedField,
                    )
                }
            } footer: {
                if let error = model.errorText {
                    Text(error)
                        .foregroundStyle(Colors.red)
                }
            }

            ListItemView(
                title: model.networkFeeTitle,
                subtitle: model.value,
                subtitleExtra: model.fiatValue,
            )
        }
        .contentMargins(.top, .scene.top, for: .scrollContent)
        .listSectionSpacing(.compact)
        .navigationTitle(model.title)
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .topBarTrailing) {
                Button("", systemImage: SystemImage.checkmark, action: model.confirm)
                    .disabled(model.isConfirmEnabled == false)
            }
        }
        .onAppear {
            focusedField = .input
        }
    }
}
