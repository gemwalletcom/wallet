// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import Testing

struct WalletBalanceTests {
    @Test
    func perpetualSumsAvailableAndReservedForTotal() {
        let balance = WalletBalance.perpetual(available: 50, reserved: 25)

        #expect(balance.total == 75)
        #expect(balance.available == 50)
    }

    @Test
    func perpetualWithZeroReserved() {
        let balance = WalletBalance.perpetual(available: 100, reserved: 0)

        #expect(balance.total == 100)
        #expect(balance.available == 100)
    }

    @Test
    func perpetualWithZeroAvailable() {
        let balance = WalletBalance.perpetual(available: 0, reserved: 0)

        #expect(balance.total == 0)
        #expect(balance.available == 0)
    }
}
