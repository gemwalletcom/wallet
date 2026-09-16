// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemConfirmLoad
@testable import Primitives
import PrimitivesTestKit
import Testing
@testable import Transfer
import TransferTestKit

struct ConfirmTransferStateTests {

    @Test
    func loadWithoutAFeeStillCarriesThePricesAndTheRecipient() throws {
        let state = try ConfirmTransferState(
            .mock(feeAssets: [.mock(asset: .mockTempoUSDC())], addressName: .mock(name: "Uniswap"), preload: nil),
            screen: .mock(),
        )

        #expect(state.preload == nil)
        #expect(state.metadata != nil)
        #expect(state.feeAssets.count == 1)
        #expect(state.addressName?.name == "Uniswap")
    }

    @Test
    func loadWithAFeeCarriesTheTransactionInput() throws {
        let state = try ConfirmTransferState(.mock(preload: .mock()), screen: .mock())

        #expect(state.preload != nil)
        #expect(state.confirmData != nil)
    }
}
