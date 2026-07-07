// Copyright (c). Gem Wallet. All rights reserved.

import Assets
import Components
import NFT
import Primitives
import Style
import SwiftUI

struct CollectionsSceneNavigationView: View {
    @Environment(\.assetsEnabler) private var assetsEnabler
    @Environment(\.priceAlertService) private var priceAlertService
    @Environment(\.activityService) private var activityService
    @Environment(\.assetSearchService) private var assetSearchService

    @State private var model: CollectionsViewModel

    init(model: CollectionsViewModel) {
        _model = State(initialValue: model)
    }

    var body: some View {
        CollectionsScene(model: model)
            .sheet(item: $model.isPresentingReceiveSelectAssetType) {
                SelectAssetSceneNavigationStack(
                    model: SelectAssetViewModel(
                        wallet: model.wallet,
                        selectType: $0,
                        searchService: assetSearchService,
                        assetsEnabler: assetsEnabler,
                        priceAlertService: priceAlertService,
                        activityService: activityService,
                    ),
                )
            }
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button(action: model.onSelectReceive) {
                        Images.System.plus
                    }
                }
            }
    }
}
