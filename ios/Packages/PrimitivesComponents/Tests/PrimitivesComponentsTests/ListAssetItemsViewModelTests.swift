// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import GemstonePrimitives
import Primitives
@testable import PrimitivesComponents
import PrimitivesTestKit
import Testing

struct ListAssetItemsViewModelTests {
    @Test
    func theWalletListHidesItsBalancesWithThePrivacyToggle() {
        let wallet = ListAssetItemsViewModel(currency: .usd).rows([.mock(asset: .mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18), price: .mock(price: 1500))])

        #expect(wallet.first?.masksBalance == true)
    }
}
