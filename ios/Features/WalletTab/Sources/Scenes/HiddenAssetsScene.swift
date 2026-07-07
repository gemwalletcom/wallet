// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI

public struct HiddenAssetsScene: View {
    @State private var model: HiddenAssetsSceneViewModel

    public init(model: HiddenAssetsSceneViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        @Bindable var preferences = model.observablePreferences

        List {
            WalletAssetsList(
                assets: model.assets,
                currencyCode: model.currencyCode,
                onHideAsset: model.onHideAsset,
                onPinAsset: model.onPinAsset,
                showBalancePrivacy: $preferences.isHideBalanceEnabled,
            )
            .listRowInsets(.assetListRowInsets)
        }
        .bindQuery(model.assetsQuery)
        .navigationTitle(model.title)
        .navigationBarTitleDisplayMode(.inline)
    }
}
