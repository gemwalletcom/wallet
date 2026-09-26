// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
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
                pay: AssetData.mock(
                    asset: Asset.mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18),
                    balance: .mock(available: availableBalance),
                ).toGem(),
                receive: nil,
                currency: Currency.usd.toGem(),
            ),
            fromAsset: fromAsset,
            onAction: {},
        )
    }
}
