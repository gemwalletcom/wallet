// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemConfirmLoad
@testable import Primitives
import PrimitivesTestKit
import Testing
@testable import Transfer
import TransferTestKit

struct ConfirmTransferStateTests {
    @Test
    func loadWithoutAFeeStillCarriesThePricesAndTheRecipient() {
        let state = ConfirmTransferState(
            .mock(feeAssets: [.mock(asset: .mockTempoUSDC())], addressName: .mock(name: "Uniswap"), fee: nil),
            screen: .mock(),
        )

        #expect(state.fee == nil)
        #expect(state.metadata != nil)
        #expect(state.feeAssets.count == 1)
        #expect(state.addressName?.name == "Uniswap")
    }

    @Test
    func loadWithAFeeCarriesTheFee() {
        let state = ConfirmTransferState(.mock(fee: .mock()), screen: .mock())

        #expect(state.fee != nil)
    }
}
