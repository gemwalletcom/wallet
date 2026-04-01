// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
import Testing

@testable import PrimitivesComponents

struct AssetViewModelTests {
    @Test
    func subtitleSymbol() {
        #expect(AssetViewModel(asset: .mock()).subtitleSymbol == "BTC")
        #expect(AssetViewModel(asset: .mockBNB()).subtitleSymbol == nil)
        #expect(AssetViewModel(asset: .mockXRP()).subtitleSymbol == nil)
        #expect(AssetViewModel(asset: .mockEthereumUSDT()).subtitleSymbol == "USDT")
    }

    @Test
    func networkFullName() {
        #expect(AssetViewModel(asset: .mockEthereum()).networkFullName == "Ethereum")
        #expect(AssetViewModel(asset: .mockEthereumUSDT()).networkFullName == "Ethereum (ERC20)")
    }
}
