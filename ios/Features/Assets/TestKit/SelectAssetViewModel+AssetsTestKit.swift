// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemAssetSelectionServiceProtocol
import class Gemstone.GemRecentActivityService
import protocol Gemstone.GemBalanceServiceProtocol
import GemstoneServicesTestKit
@testable import Assets
import GemstoneServices
import Components
import Foundation
import GemstonePrimitivesTestKit
import GemstonePrimitives
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
        service: any GemAssetSelectionServiceProtocol = GemAssetSelectionServiceMock(),
    ) -> SelectAssetViewModel {
        let model = SelectAssetViewModel(
            wallet: wallet,
            selectType: selectType,
            service: service,
            recentAssetsService: GemRecentActivityService(store: GemstoneRecentActivityStore(store: .mock()), session: .mock()),
        )
        model.assetsQuery.value = assets
        model.state = state
        return model
    }
}
