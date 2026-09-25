// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemSwapQuoteServiceProtocol
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
@testable import Store
@testable import Swap

public extension SwapSceneViewModel {
    static func mock(
        service: any GemSwapQuoteServiceProtocol = GemSwapQuoteServiceMock(),
        pairSelector: SwapPairSelectorViewModel = SwapPairSelectorViewModel(fromAssetId: .mock(chain: .ethereum), toAssetId: nil),
    ) -> SwapSceneViewModel {
        let model = SwapSceneViewModel(
            service: service,
            input: SwapInput(
                wallet: .mock(accounts: [.mock(chain: .ethereum)]),
                pairSelector: pairSelector,
            ),
        )
        model.fromAssetQuery.value = .mock(asset: .mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18), balance: .mock())
        model.toAssetQuery.value = .mock(asset: .mock(id: .mock(chain: .ethereum, tokenId: "0xdAC17F958D2ee523a2206206994597C13D831ec7"), name: "Tether", symbol: "USDT", decimals: 6, type: .erc20))
        model.amountInputModel.text = "1"
        model.onChangeFromValue("", "1")
        return model
    }
}
