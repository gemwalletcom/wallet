// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemAmountService
import GemstonePrimitives
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
@testable import Store
import Transfer

public extension AmountSceneViewModel {
    static func mock(
        type: AmountType = .transfer(recipient: .mock()),
        assetData: AssetData = .mock(balance: .mock()),
    ) -> AmountSceneViewModel {
        let model = AmountSceneViewModel(
            input: AmountInput(type: type, asset: assetData.asset),
            wallet: .mock(),
            service: GemAmountServiceMock(builder: GemAmountService.mock()),
            onTransferAction: { _ in },
        )
        model.assetQuery.value = assetData
        model.onChangeAssetBalance(assetData, assetData)
        return model
    }
}
