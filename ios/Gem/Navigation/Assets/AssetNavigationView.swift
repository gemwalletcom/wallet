// Copyright (c). Gem Wallet. All rights reserved.

import Assets
import Components
import Foundation
import InfoSheet
import Localization
import PriceAlerts
import Primitives
import PrimitivesComponents
import Store
import SwiftUI

struct AssetNavigationView: View {
    @State private var model: AssetSceneViewModel

    init(model: AssetSceneViewModel) {
        _model = State(initialValue: model)
    }

    var body: some View {
        AssetScene(
            model: model,
        )
        .bindQuery(model.assetQuery, model.bannersQuery, model.transactionsQuery)
        .toolbar {
            ToolbarItemGroup(placement: .topBarTrailing) {
                Button(action: model.onTogglePriceAlert) {
                    model.priceAlertsImage
                }

                AdaptiveActionMenu(
                    title: model.title,
                    items: model.menuItems,
                    label: { model.optionsImage },
                )
            }
        }
        .toast(message: $model.isPresentingToastMessage)
        .sheet(item: $model.isPresentingAssetSheet) {
            switch $0 {
            case let .info(type):
                InfoSheetScene(type: type)
            case let .transfer(data):
                ConfirmTransferNavigationStack(
                    wallet: model.walletModel.wallet,
                    transferData: data,
                    onComplete: model.onTransferComplete,
                )
            case .share:
                ShareSheet(activityItems: [model.shareAssetUrl.absoluteString])
            case let .url(url):
                SFSafariView(url: url)
            }
        }
    }
}
