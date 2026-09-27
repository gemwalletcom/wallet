// Copyright (c). Gem Wallet. All rights reserved.

import Components
import func Gemstone.chainRow
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct AssetsFilterScene: View {
    @Environment(\.dismiss) var dismiss
    @Bindable var model: SelectAssetSceneViewModel

    @State private var isPresentingChains: Bool = false

    public init(model: SelectAssetSceneViewModel) {
        self.model = model
    }

    public var body: some View {
        List {
            SelectFilterView(
                typeModel: model.chainsTypeModel,
                action: onSelectChainsFilter,
            )

            if model.filterView.showsBalanceToggle {
                ListItemToggleView(
                    isOn: $model.hasBalance,
                    title: Localized.Filter.hasBalance,
                    imageStyle: .settings(assetImage: .image(Images.Filters.balance)),
                )
            }
        }
        .contentMargins(.top, .scene.top, for: .scrollContent)
        .listStyle(.insetGrouped)
        .navigationTitle(Localized.Filter.title)
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            if model.filterView.isFiltered {
                ToolbarItem(placement: .cancellationAction) {
                    Button(Localized.Filter.clear, action: model.onClearFilters)
                        .bold()
                }
            }
        }
        .toolbarDismissItem(type: .done, placement: .primaryAction)
        .sheet(isPresented: $isPresentingChains) {
            SelectableSheet(
                model: model.networksModel,
                onFinishSelection: onFinishSelection(value:),
                listContent: { ChainView(model: chainRow(chain: $0.rawValue)) },
            )
        }
    }
}

// MARK: - Actions

extension AssetsFilterScene {
    private func onFinishSelection(value: SelectionResult<Chain>) {
        if model.onFinishChainsSelection(value) {
            dismiss()
        }
    }

    private func onSelectChainsFilter() {
        isPresentingChains.toggle()
    }
}
