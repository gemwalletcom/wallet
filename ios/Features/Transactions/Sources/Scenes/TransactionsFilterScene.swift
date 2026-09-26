// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct TransactionsFilterScene: View {
    @Environment(\.dismiss) private var dismiss
    @Binding private var model: TransactionsFilterSceneViewModel

    public init(model: Binding<TransactionsFilterSceneViewModel>) {
        _model = model
    }

    public var body: some View {
        List {
            SelectFilterView(
                typeModel: model.chainsFilter.typeModel,
                action: model.onSelectChainsFilter,
            )
            SelectFilterView(
                typeModel: model.transactionTypesFilter.typeModel,
                action: model.onSelectTypesFilter,
            )
        }
        .contentMargins(.top, .scene.top, for: .scrollContent)
        .listStyle(.insetGrouped)
        .navigationTitle(model.title)
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            if model.isAnyFilterSpecified {
                ToolbarItem(placement: .cancellationAction) {
                    Button(model.clear, action: onSelectClear)
                        .bold()
                }
            }
            ToolbarItem(placement: .primaryAction) {
                Button(model.done, action: onSelectDone)
                    .bold()
            }
        }
        .sheet(isPresented: $model.isPresentingChains) {
            SelectableSheet(
                model: model.networksModel,
                onFinishSelection: {
                    if model.onFinishChainsSelection($0) {
                        dismiss()
                    }
                },
                listContent: { ChainView(model: ChainViewModel(chain: $0)) },
            )
        }
        .sheet(isPresented: $model.isPresentingTypes) {
            SelectableSheet(
                model: model.typesModel,
                onFinishSelection: {
                    if model.onFinishTypesSelection($0) {
                        dismiss()
                    }
                },
                listContent: {
                    ListItemView(model: $0.listItem)
                },
            )
        }
    }
}

// MARK: - Actions

extension TransactionsFilterScene {
    private func onSelectClear() {
        model.chainsFilter.selectedChains = []
        model.transactionTypesFilter.selectedTypes = []
    }

    private func onSelectDone() {
        dismiss()
    }
}
