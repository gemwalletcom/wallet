// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemCurrencyRow
import PrimitivesComponents
import SwiftUI

public struct CurrencyScene: View {
    @Environment(\.dismiss) private var dismiss
    @State private var model: CurrencySceneViewModel

    public init(model: CurrencySceneViewModel) {
        self.model = model
    }

    public var body: some View {
        let list = model.list
        return List(list?.sections ?? [], id: \.kind) { section in
            Section(section.kind.title) {
                ForEach(section.rows, id: \.currency) { row in
                    ListItemSelectionView(
                        title: row.title,
                        value: row.currency,
                        selection: row.isSelected ? row.currency : nil,
                    ) { _ in
                        onSelectCurrency(row)
                    }
                }
            }
        }
        .listSectionSpacing(.compact)
        .searchable(text: $model.searchQuery, placement: .navigationBarDrawer(displayMode: .always))
        .autocorrectionDisabled(true)
        .textInputAutocapitalization(.never)
        .scrollDismissesKeyboard(.interactively)
        .overlay {
            if case let .empty(state) = list?.phase {
                EmptyContentView(model: EmptyStateViewModel(state: state))
            }
        }
        .task(id: model.searchQuery) {
            await model.refreshSections()
        }
        .navigationTitle(model.title)
        .alertSheet($model.isPresentingAlertMessage)
    }
}

// MARK: - Actions

extension CurrencyScene {
    private func onSelectCurrency(_ row: GemCurrencyRow) {
        guard !row.isSelected else { return }

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
