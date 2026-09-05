// Copyright (c). Gem Wallet. All rights reserved.

import Components
import InfoSheet
import PrimitivesComponents
import Style
import SwiftUI

public struct SwapSlippageScene: View {
    private enum Field: Hashable {
        case slippage
    }

    @Environment(\.dismiss) private var dismiss

    @State private var model: SwapSlippageViewModel
    @FocusState private var focusedField: Field?

    public init(model: SwapSlippageViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        NavigationStack {
            List {
                Section {
                    Toggle(model.autoTitle, isOn: $model.isAuto)
                        .toggleStyle(AppToggleStyle())
                } footer: {
                    Text(model.autoDescription)
                }

                if !model.isAuto {
                    Section {
                        HStack(spacing: .small) {
                            Text(model.title)
                                .lineLimit(1)
                                .fixedSize(horizontal: true, vertical: false)
                            InfoButton { model.onSelectInfo() }
                            SuffixTextField(
                                placeholder: model.placeholder,
                                suffix: "%",
                                sanitizer: model.sanitize,
                                text: $model.inputModel.text,
                                field: Field.slippage,
                                focusedField: $focusedField,
                            )
                        }
                    } footer: {
                        if let error = model.errorText {
                            Text(.init(error))
                                .foregroundStyle(Colors.red)
                        } else if let warning = model.warningText {
                            Text(warning)
                                .foregroundStyle(Colors.red)
                        }
                    }
                }
            }
            .navigationTitle(model.title)
            .navigationBarTitleDisplayMode(.inline)
            .listSectionSpacing(.compact)
            .contentMargins([.top], .small, for: .scrollContent)
            .safeAreaView {
                if focusedField == .slippage {
                    SuggestionsAccessoryView(
                        suggestions: model.suggestions,
                        onSelect: { model.onSelect(suggestion: $0) },
                        onDone: { focusedField = nil },
                    )
                    .padding(.small)
                }
            }
            .toolbar {
                ToolbarItem(placement: .topBarTrailing) {
                    Button("", systemImage: SystemImage.checkmark) {
                        model.confirm()
                        dismiss()
                    }
                    .disabled(model.isConfirmEnabled == false)
                }
            }
            .onChange(of: model.isAuto) { _, isAuto in
                focusedField = isAuto ? nil : .slippage
            }
            .onAppear {
                if !model.isAuto {
                    focusedField = .slippage
                }
            }
            .sheet(item: $model.infoSheet) {
                InfoSheetScene(type: $0)
            }
        }
    }
}
