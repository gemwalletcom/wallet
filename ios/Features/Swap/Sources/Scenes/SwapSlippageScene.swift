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

    @State private var model: SwapSlippageSceneViewModel
    @FocusState private var focusedField: Field?

    public init(model: SwapSlippageSceneViewModel) {
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
                                placeholder: model.viewState.placeholder,
                                suffix: "%",
                                text: $model.input,
                                field: Field.slippage,
                                focusedField: $focusedField,
                            )
                        }
                    } footer: {
                        if let footer = model.viewState.footer?.text {
                            Text(.init(footer))
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
                        suggestions: model.viewState.suggestions,
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
                    .disabled(model.viewState.allowsConfirm == false)
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
                InfoSheetScene(sheet: $0)
            }
        }
    }
}
