// Copyright (c). Gem Wallet. All rights reserved.

@testable import Assets
import Components
import Foundation
import protocol Gemstone.GemAssetSelectionServiceProtocol
import protocol Gemstone.GemBalanceServiceProtocol
import class Gemstone.GemPaymentService
import class Gemstone.GemRecentActivityService
import GemstonePrimitives
import GemstonePrimitivesTestKit
import GemstoneServices
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
@testable import Store

public extension SelectAssetSceneViewModel {
    @MainActor
    static func mock(
        wallet: Wallet = .mock(),
        selectType: SelectAssetType = .manage,
        assets: [AssetData] = [],
        state: StateViewType<[AssetBasic]> = .noData,
        service: any GemAssetSelectionServiceProtocol = GemAssetSelectionServiceMock(),
        chains: [Chain] = [],
    ) -> SelectAssetSceneViewModel {
        let model = SelectAssetSceneViewModel(
            wallet: wallet,
            selectType: selectType,
            service: service,
            paymentService: GemPaymentService.mock(),
            recentAssetsService: GemRecentActivityService(store: GemstoneRecentActivityStore(store: .mock()), session: .mock()),
            chains: chains,
        )
        model.assetsQuery.value = assets
        model.state = state
        return model
    }
}
