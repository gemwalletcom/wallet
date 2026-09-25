// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
@testable import PrimitivesComponents
import PrimitivesTestKit
import Testing

struct AssetViewModelTests {
    @Test
    func subtitleSymbol() {
        #expect(AssetViewModel(asset: .mock(name: "Bitcoin", symbol: "BTC", decimals: 8)).subtitleSymbol == "BTC")
        #expect(AssetViewModel(asset: .mock(id: .mock(chain: .smartChain), name: "BNB", symbol: "BNB", decimals: 18)).subtitleSymbol == nil)
        #expect(AssetViewModel(asset: .mock(id: .mock(chain: .xrp), name: "XRP", symbol: "XRP", decimals: 8)).subtitleSymbol == nil)
        #expect(AssetViewModel(asset: .mock(id: .mock(chain: .ethereum, tokenId: "0xdAC17F958D2ee523a2206206994597C13D831ec7"), name: "Tether", symbol: "USDT", decimals: 6, type: .erc20)).subtitleSymbol == "USDT")
    }

    @Test
    func networkFullName() {
        #expect(AssetViewModel(asset: .mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18)).networkFullName == "Ethereum")
        #expect(AssetViewModel(asset: .mock(id: .mock(chain: .ethereum, tokenId: "0xdAC17F958D2ee523a2206206994597C13D831ec7"), name: "Tether", symbol: "USDT", decimals: 6, type: .erc20)).networkFullName == "Ethereum (ERC20)")
    }
}
