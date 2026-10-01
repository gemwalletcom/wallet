// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.PerpetualBalance
import func Gemstone.perpetualBalanceHeader
import Primitives
import PrimitivesComponents
import Testing

struct PerpetualBalanceHeaderTests {
    private func model(available: Double, reserved: Double) -> ValueHeader {
        perpetualBalanceHeader(
            balance: PerpetualBalance(available: available, reserved: reserved, withdrawable: available),
            walletType: WalletType.multicoin.toGem(),
        ).valueHeader
    }

    @Test
    func theHeaderShowsTheTotalAndTheAvailableBalance() {
        let header = model(available: 300, reserved: 950)

        #expect(header.title.contains("1,250"))
        #expect(header.subtitle?.contains("300") == true)
    }
}
