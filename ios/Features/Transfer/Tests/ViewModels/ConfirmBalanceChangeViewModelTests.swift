// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Localization
@testable import Primitives
import PrimitivesTestKit
import Style
import Testing
@testable import Transfer

struct ConfirmBalanceChangeViewModelTests {
    @Test
    func balanceChange() {
        let solana = Asset.mock(id: .mockSolana(), name: "Solana", symbol: "SOL", decimals: 9, type: .native)
        let negative = ConfirmBalanceChangeViewModel(balanceChange: SimulationAssetChange(assetId: solana.id, value: BigInt(-100_005_000), decimals: 9, name: "Solana", symbol: "SOL"))
        let positive = ConfirmBalanceChangeViewModel(balanceChange: SimulationAssetChange(assetId: solana.id, value: BigInt(100_005_000), decimals: 9, name: "Solana", symbol: "SOL"))

        #expect(negative.assetTitle == "Solana")
        #expect(negative.amount == "-0.100005 SOL")
        #expect(positive.amount == "+0.100005 SOL")
        #expect(negative.isUnknown == false)
        #expect(negative.color == Colors.red)
        #expect(positive.color == Colors.green)
    }

    @Test
    func unknownBalanceChange() {
        let assetId = AssetId(chain: .solana, tokenId: "MissingMint111111111111111111111111111111111")
        let unknown = ConfirmBalanceChangeViewModel(balanceChange: SimulationAssetChange(assetId: assetId, value: BigInt(-42), decimals: 2, name: nil, symbol: nil))

        #expect(unknown.isUnknown)
        #expect(unknown.assetTitle == Localized.Errors.unknown)
        #expect(unknown.amount == "-0.42")
        #expect(unknown.color == Colors.red)
    }
}
