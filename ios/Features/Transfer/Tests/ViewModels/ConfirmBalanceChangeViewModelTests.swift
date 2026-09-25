// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemAssetConfigService
import struct Gemstone.GemSimulationBalanceChange
import enum Gemstone.GemValueTone
@testable import GemstonePrimitives
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesTestKit
import Style
import Testing
@testable import Transfer

struct ConfirmBalanceChangeViewModelTests {
    @Test
    func balanceChange() {
        let solana = Asset.mockSolana()
        let change = { (value: Double, tone: GemValueTone) in
            ConfirmBalanceChangeViewModel(balanceChange: GemSimulationBalanceChange(
                asset: solana.toGem(),
                icon: GemAssetConfigService.shared.assetIcon(assetId: solana.id.identifier),
                amount: .mock(value: value, unit: .symbol(symbol: "SOL"), display: .number(precision: .fraction(min: 0, max: 32)), tone: tone, exact: "1.5"),
            ))
        }
        let negative = change(-1.5, .negative)
        let positive = change(1.5, .positive)

        #expect(negative.assetTitle == "Solana")
        #expect(negative.amount.text == "-1.5 SOL")
        #expect(positive.amount.text == "+1.5 SOL")
        #expect(negative.amount.style.color == Colors.red)
        #expect(positive.amount.style.color == Colors.green)
    }
}
