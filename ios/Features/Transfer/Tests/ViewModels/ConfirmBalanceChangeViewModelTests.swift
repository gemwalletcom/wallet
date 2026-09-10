// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import struct Gemstone.GemSimulationBalanceChange
@testable import GemstonePrimitives
import Primitives
import PrimitivesTestKit
import Style
import Testing
@testable import Transfer

struct ConfirmBalanceChangeViewModelTests {
    @Test
    func balanceChange() {
        let solana = Asset.mock(id: .mockSolana(), name: "Solana", symbol: "SOL", decimals: 9, type: .native)
        let negative = ConfirmBalanceChangeViewModel(balanceChange: GemSimulationBalanceChange(asset: solana.map(), value: BigInt(-1_500_000_000), sign: .outgoing))
        let positive = ConfirmBalanceChangeViewModel(balanceChange: GemSimulationBalanceChange(asset: solana.map(), value: BigInt(1_500_000_000), sign: .incoming))

        #expect(negative.assetTitle == "Solana")
        #expect(negative.amount.text == "-1.5 SOL")
        #expect(positive.amount.text == "+1.5 SOL")
        #expect(negative.amount.style.color == Colors.red)
        #expect(positive.amount.style.color == Colors.green)
    }
}
