// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import struct Gemstone.GemAssetBalance
import struct Gemstone.GemSwapAssetData
import struct Gemstone.GemSwapSession
import GemstonePrimitives
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesTestKit
@testable import Swap

extension SwapButtonViewModel {
    static func mock(
        session: GemSwapSession = .mock(),
        availableBalance: BigInt = .zero,
        fromAsset: AssetData? = .mock(),
    ) -> SwapButtonViewModel {
        SwapButtonViewModel(
            state: session.viewState(
                pay: GemSwapAssetData(
                    asset: Asset.mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18).toGem(),
                    balance: GemAssetBalance(Balance.mock(available: availableBalance), assetId: Asset.mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18).id, isActive: true),
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
