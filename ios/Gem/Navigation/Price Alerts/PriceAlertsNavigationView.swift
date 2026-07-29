// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Localization
import PriceAlerts
import Primitives
import Style
import SwiftUI

struct PriceAlertsNavigationView: View {
    @Environment(\.walletService) private var walletService
    @Environment(\.viewModelFactory) private var viewModelFactory

    @State private var isPresentingAddAsset: Bool = false
    @State private var isPresentingToastMessage: ToastMessage?

    let model: PriceAlertsSceneViewModel

    var body: some View {
        PriceAlertsScene(model: model)
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button {
                        isPresentingAddAsset = true
                    } label: {
                        Images.System.plus
                    }
                }
            }
            .sheet(isPresented: $isPresentingAddAsset) {
                AddAssetPriceAlertsNavigationStack(
                    selectAssetModel: viewModelFactory.selectAssetScene(
                        wallet: walletService.currentWallet!,
                        selectType: .priceAlert,
                        selectAssetAction: onSelectAsset,
                    ),
                )
            }
            .toast(message: $isPresentingToastMessage)
    }

    private func onSelectAsset(asset: Asset) {
        isPresentingAddAsset = false
        isPresentingToastMessage = .priceAlert(for: asset.name, enabled: true)
    }
}
