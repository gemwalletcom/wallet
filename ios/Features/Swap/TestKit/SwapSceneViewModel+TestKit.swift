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
        pairSelector: SwapPairSelectorViewModel = SwapPairSelectorViewModel(fromAssetId: .mockEthereum(), toAssetId: nil),
    ) -> SwapSceneViewModel {
        let model = SwapSceneViewModel(
            service: service,
            input: SwapInput(
                wallet: .mock(accounts: [.mock(chain: .ethereum)]),
                pairSelector: pairSelector,
            ),
        )
        model.fromAssetQuery.value = .mock(asset: .mockEthereum(), balance: .mock())
        model.toAssetQuery.value = .mock(asset: .mockEthereumUSDT())
        model.amountInputModel.text = "1"
        model.onChangeFromValue("", "1")
        return model
    }
}
