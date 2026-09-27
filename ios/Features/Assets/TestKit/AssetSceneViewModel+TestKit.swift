// Copyright (c). Gem Wallet. All rights reserved.

import Assets
import protocol Gemstone.GemAssetDetailsServiceProtocol
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
@testable import Store

public extension AssetSceneViewModel {
    @MainActor
    static func mock(
        _ assetData: AssetData = .mock(),
        service: any GemAssetDetailsServiceProtocol = GemAssetDetailsServiceMock(),
    ) -> AssetSceneViewModel {
        let model = AssetSceneViewModel(
            service: service,
            preferences: .mock(),
            input: AssetSceneInput(
                wallet: .mock(),
                asset: assetData.asset,
            ),
            isPresentingSelectedAssetInput: .constant(.none),
        )
        model.assetQuery.value = ChainAssetData(
            assetData: assetData,
            feeAssetData: .with(asset: assetData.asset.chain.asset),
        )
        return model
    }
}
