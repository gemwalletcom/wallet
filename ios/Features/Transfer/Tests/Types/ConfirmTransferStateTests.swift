// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemConfirmLoad
@testable import Primitives
import PrimitivesTestKit
import Testing
@testable import Transfer
import TransferTestKit

struct ConfirmTransferStateTests {
    @Test
    func loadWithoutAFeeStillCarriesThePrices() {
        let state = ConfirmTransferState(
            .mock(feeAssets: [.mock(asset: Primitives.Asset.mock(id: .mock(chain: .tempo, tokenId: "0x20C000000000000000000000b9537d11c60E8b50"), name: "Bridged USDC", symbol: "USDC.e", decimals: 6, type: .tip20).toGem())], fee: nil),
            screen: .mock(),
        )

        #expect(state.fee == nil)
        #expect(state.metadata != nil)
        #expect(state.feeAssets.count == 1)
    }

    @Test
    func loadWithAFeeCarriesTheFee() {
        let state = ConfirmTransferState(.mock(fee: .mock()), screen: .mock())

        #expect(state.fee != nil)
    }
}
