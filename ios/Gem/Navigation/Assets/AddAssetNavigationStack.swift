// Copyright (c). Gem Wallet. All rights reserved.

import Assets
import GemstoneServices
import Localization
import Primitives
import Style
import SwiftUI

struct AddAssetNavigationStack: View {
    let wallet: Wallet
    @Environment(\.gatewayService) private var gatewayService
    @Environment(\.explorerService) private var explorerService
    @Environment(\.assetStore) private var assetStore
    @Environment(\.assetsEnabler) private var assetsEnabler
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationStack {
            AddAssetScene(
                model: AddAssetSceneViewModel(
                    wallet: wallet,
                    gatewayService: gatewayService,
                    explorerService: explorerService,
                ),
                action: addAsset,
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

extension AddAssetNavigationStack {
    private func addAsset(_ asset: Asset) {
        Task {
            do {
                try assetStore.add(assets: [asset.defaultBasic])
                try await assetsEnabler.enableAssets(wallet: wallet, assetIds: [asset.id], enabled: true)
            } catch {
                debugLog("AddAssetNavigationStack add asset error: \(error)")
            }
        }
        dismiss()
    }
}
