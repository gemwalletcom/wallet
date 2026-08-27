// Copyright (c). Gem Wallet. All rights reserved.

import GemstoneServicesTestKit
@testable import Assets
import GemstoneServices
import Components
import Foundation
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesTestKit
@testable import Store

public extension SelectAssetViewModel {
    @MainActor
    static func mock(
        wallet: Wallet = .mock(),
        selectType: SelectAssetType = .manage,
        assets: [AssetData] = [],
        state: StateViewType<[AssetBasic]> = .noData,
        assetsEnabler: any AssetsEnabler = .mock(),
    ) -> SelectAssetViewModel {
        let model = SelectAssetViewModel(
            wallet: wallet,
            selectType: selectType,
            searchService: GemSearchServiceMock(),
            assetsEnabler: assetsEnabler,
            priceAlertService: GemPriceAlertServiceMock(),
            recentActivityStore: .mock(),
        )
        model.assetsQuery.value = assets
        model.state = state
        return model
    }
}
