// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Style
import SwiftUI

public struct SwapSlippageScene: View {
    @Environment(\.dismiss) private var dismiss

    @State private var model: SwapSlippageViewModel

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
                        Picker("", selection: $model.selectedBps) {
                            ForEach(SwapSlippageViewModel.options, id: \.self) { bps in
                                Text(SwapSlippageSuggestion(bps: bps).title)
                                    .tag(bps)
                            }
                        }
                        .pickerStyle(.segmented)
                        .onChange(of: model.selectedBps) { model.apply() }
                        ListItemView(field: model.selectedField)
                    } footer: {
                        if let warning = model.warningText {
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
            .toolbar {
                ToolbarItem(placement: .topBarTrailing) {
                    Button("", systemImage: SystemImage.checkmark) { dismiss() }
                }
            }
            .onChange(of: model.isAuto) { model.apply() }
        }
    }
}
