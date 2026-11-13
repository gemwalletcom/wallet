// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemListItem
import GemstonePrimitives
import PrimitivesComponents
import SwiftUI

public struct CurrencyScene: View {
    @Environment(\.dismiss) private var dismiss
    @State private var model: CurrencySceneViewModel

    public init(model: CurrencySceneViewModel) {
        self.model = model
    }

    public var body: some View {
        GemGroupListView(list: model.list, onSelect: onSelect)
            .listSectionSpacing(.compact)
            .searchable(text: $model.searchQuery, placement: .navigationBarDrawer(displayMode: .always))
            .autocorrectionDisabled(true)
            .textInputAutocapitalization(.never)
            .scrollDismissesKeyboard(.interactively)
            .task(id: model.searchQuery) {
                await model.refreshSections()
            }
            .navigationTitle(model.title)
            .alertSheet($model.isPresentingAlertMessage)
    }
}

// MARK: - Actions

extension CurrencyScene {
    private func onSelect(_ item: GemListItem) {
        guard case let .currency(row) = item, !row.isSelected else { return }

        Task {
            do {
                try await model.setCurrency(row.currency)
                dismiss()
            } catch {
                model.isPresentingAlertMessage = AlertMessage(error: error)
            }
        }
    }
}
