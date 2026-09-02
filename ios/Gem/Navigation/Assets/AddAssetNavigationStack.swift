// Copyright (c). Gem Wallet. All rights reserved.

import Assets
import Localization
import Primitives
import Style
import SwiftUI

struct AddAssetNavigationStack: View {
    let wallet: Wallet
    @Environment(\.viewModelFactory) private var viewModelFactory
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationStack {
            AddAssetScene(
                model: viewModelFactory.addAssetScene(wallet: wallet),
                onComplete: { dismiss() },
            )
            .navigationTitle(Localized.Settings.Networks.title)
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .topBarLeading) {
                    Button("", systemImage: SystemImage.xmark) {
                        dismiss()
                    }
                }
            }
        }
    }
}
