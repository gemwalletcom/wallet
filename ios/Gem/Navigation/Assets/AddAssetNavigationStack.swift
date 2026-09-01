// Copyright (c). Gem Wallet. All rights reserved.

import Assets
import GemstonePrimitives
import GemstoneServices
import Localization
import Primitives
import Style
import SwiftUI

struct AddAssetNavigationStack: View {
    let wallet: Wallet
    @Environment(\.gatewayService) private var gatewayService
    @Environment(\.assetConfig) private var assetConfig
    @Environment(\.chainService) private var chainService
    @Environment(\.explorerService) private var explorerService
    @Environment(\.assetsService) private var assetsService
    @Environment(\.balanceService) private var balanceService
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationStack {
            AddAssetScene(
                model: AddAssetSceneViewModel(
                    wallet: wallet,
                    gatewayService: gatewayService,
                    explorerService: explorerService,
                    assetConfig: assetConfig,
                ),
                chainService: chainService,
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
                let asset = try await assetsService.ensureTokenAsset(for: asset.id)
                try await balanceService.setAssetsEnabled(wallet: wallet, assetIds: [asset.id], enabled: true)
            } catch {
                debugLog("AddAssetNavigationStack add asset error: \(error)")
            }
        }
        dismiss()
    }
}
