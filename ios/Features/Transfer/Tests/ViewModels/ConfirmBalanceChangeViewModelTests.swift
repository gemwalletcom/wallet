// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
@testable import Primitives
import PrimitivesTestKit
import Style
import Testing
@testable import Transfer

struct ConfirmBalanceChangeViewModelTests {
    @Test
    func balanceChange() {
        let solana = Asset.mock(id: .mockSolana(), name: "Solana", symbol: "SOL", decimals: 9, type: .native)
        let negative = ConfirmBalanceChangeViewModel(balanceChange: SimulationAssetChange(asset: solana, value: BigInt(-100_005_000)))
        let positive = ConfirmBalanceChangeViewModel(balanceChange: SimulationAssetChange(asset: solana, value: BigInt(100_005_000)))

        #expect(negative.title == "-0.100005 SOL")
        #expect(positive.title == "+0.100005 SOL")
        #expect(negative.color == Colors.red)
        #expect(positive.color == Colors.green)
    }
}
