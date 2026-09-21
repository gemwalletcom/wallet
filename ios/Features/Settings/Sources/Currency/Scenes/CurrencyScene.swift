// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemCurrencyRow
import SwiftUI

public struct CurrencyScene: View {
    @Environment(\.dismiss) private var dismiss
    @State private var model: CurrencySceneViewModel

    public init(model: CurrencySceneViewModel) {
        self.model = model
    }

    public var body: some View {
        let sections = model.sections
        List(sections, id: \.kind) { section in
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
            if sections.isEmpty {
                ContentUnavailableView.search(text: model.searchQuery)
            }
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
