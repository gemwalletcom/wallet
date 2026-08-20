// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import SwiftUI

public struct AppearanceScene: View {
    @Environment(\.dismiss) private var dismiss
    @State private var model: AppearanceViewModel

    public init(model: AppearanceViewModel) {
        self.model = model
    }

    public var body: some View {
        List(model.options) { option in
            ListItemSelectionView(
                title: option.title,
                value: option,
                selection: model.appearance,
            ) {
                onSelect($0)
            }
        }
        .listSectionSpacing(.compact)
        .navigationTitle(model.title)
    }
}

// MARK: - Actions

extension AppearanceScene {
    private func onSelect(_ appearance: Appearance) {
        model.onSelect(appearance: appearance)
        dismiss()
    }
}
