// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import struct Gemstone.GemAssetBalance
import struct Gemstone.GemSwapAssetData
import struct Gemstone.GemSwapRequest
import struct Gemstone.GemSwapSession
import GemstonePrimitives
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
@testable import Swap

extension SwapButtonViewModel {
    static func mock(
        session: GemSwapSession = .mock(),
        availableBalance: BigInt = BigInt(GemSwapRequest.mock.value),
        fromAsset: AssetData? = .mock(),
    ) -> SwapButtonViewModel {
        SwapButtonViewModel(
            state: session.viewState(
                pay: GemSwapAssetData(
                    asset: Asset.mockEthereum().toGem(),
                    balance: GemAssetBalance(Balance.mock(available: availableBalance), assetId: Asset.mockEthereum().id, isActive: true),
                    price: nil,
                ),
                receive: nil,
                currency: Currency.usd.toGem(),
            ),
            fromAsset: fromAsset,
            onAction: {},
        )
    }
}
