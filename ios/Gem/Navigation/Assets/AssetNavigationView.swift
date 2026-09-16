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
        let details = model.details
        return AssetScene(model: model)
        .bindQuery(model.assetQuery, model.bannersQuery, model.transactionsQuery)
        .toolbar {
            ToolbarItemGroup(placement: .topBarTrailing) {
                Button(action: model.onTogglePriceAlert) {
                    model.priceAlertsImage(details)
                }

                AdaptiveActionMenu(
                    title: details.title,
                    items: model.menuItems(details),
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
                    wallet: model.wallet,
                    transferData: data,
                    onComplete: model.onTransferComplete,
                )
            case .share:
                ShareSheet(activityItems: [model.shareAssetUrl(details).absoluteString])
            case let .url(url):
                SFSafariView(url: url)
            }
        }
    }
}
