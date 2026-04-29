// Copyright (c). Gem Wallet. All rights reserved.

import ActivityServiceTestKit
@testable import Assets
import AssetsServiceTestKit
import BalanceServiceTestKit
import Components
import Foundation
import PriceAlertServiceTestKit
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
    ) -> SelectAssetViewModel {
        let model = SelectAssetViewModel(
            wallet: wallet,
            selectType: selectType,
            searchService: .mock(),
            assetsEnabler: .mock(),
            priceAlertService: .mock(),
            activityService: .mock(),
        )
        model.assetsQuery.value = assets
        model.state = state
        return model
    }
}
