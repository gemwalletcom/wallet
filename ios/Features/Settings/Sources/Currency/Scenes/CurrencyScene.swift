// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import SwiftUI

public struct CurrencyScene: View {
    @Environment(\.dismiss) private var dismiss
    @State private var model: CurrencySceneViewModel

    public init(model: CurrencySceneViewModel) {
        self.model = model
    }

    public var body: some View {
        List(model.list) { section in
            Section(section.section) {
                ForEach(section.values) {
                    ListItemSelectionView(
                        title: $0.title,
                        titleExtra: .none,
                        titleTag: .none,
                        titleTagType: .none,
                        subtitle: .none,
                        subtitleExtra: .none,
                        value: $0.value.currency,
                        selection: model.currency,
                    ) {
                        onSelectCurrency($0)
                    }
                }
            }
        }
        .listSectionSpacing(.compact)
        .navigationTitle(model.title)
    }
}

// MARK: - Actions

extension CurrencyScene {
    private func onSelectCurrency(_ currency: Currency) {
        guard currency != model.currency else { return }

        do {
            try model.setCurrency(currency)
            dismiss()
        } catch {
            return
        }
    }
}
