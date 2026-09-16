// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemPerpetualPositionAction
import GemstonePrimitives
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
import Perpetuals
import Primitives
import PrimitivesTestKit

public extension PerpetualSceneViewModel {
    static func mock(
        service: GemPerpetualDetailsServiceMock = GemPerpetualDetailsServiceMock(),
        asset: Asset = .mock(),
        onTransferData: TransferDataAction = nil,
        onPerpetualPosition: ((GemPerpetualPositionAction) -> Void)? = nil,
    ) -> PerpetualSceneViewModel {
        PerpetualSceneViewModel(
            wallet: .mock(),
            asset: asset,
            service: service,
            observerService: PerpetualObserverMock(),
            onTransferData: onTransferData,
            onPerpetualPosition: onPerpetualPosition,
        )
    }
}
