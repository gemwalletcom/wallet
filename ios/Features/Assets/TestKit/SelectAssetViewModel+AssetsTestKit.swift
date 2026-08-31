// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemAssetConfigService
import protocol Gemstone.GemBalanceServiceProtocol
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
        balanceService: any GemBalanceServiceProtocol = .mock(),
    ) -> SelectAssetViewModel {
        let model = SelectAssetViewModel(
            wallet: wallet,
            selectType: selectType,
            searchService: GemSearchServiceMock(),
            balanceService: balanceService,
            priceAlertService: GemPriceAlertServiceMock(),
            recentAssetsService: RecentAssetsService(store: .mock()),
            preferencesService: GemPreferencesServiceMock(),
            assetConfig: GemAssetConfigService(),
        )
        model.assetsQuery.value = assets
        model.state = state
        return model
    }
}
