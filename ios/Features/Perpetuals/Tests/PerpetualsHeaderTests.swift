// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.PerpetualBalance
import func Gemstone.perpetualBalanceHeader
@testable import Perpetuals
import Primitives
import PrimitivesComponents
import Testing

struct PerpetualsHeaderTests {
    private func model(available: Double, reserved: Double, withdrawable: Double? = nil, walletType: WalletType = .multicoin) -> ValueHeader {
        perpetualBalanceHeader(
            balance: PerpetualBalance(available: available, reserved: reserved, withdrawable: withdrawable ?? available),
            walletType: walletType.toGem(),
        ).valueHeader
    }

    @Test
    func theHeaderShowsTheTotalAndTheAvailableBalance() {
        let header = model(available: 300, reserved: 950)

        #expect(header.title.contains("1,250"))
        #expect(header.subtitle?.contains("300") == true)
    }

    @Test
    func withdrawIsOfferedOnlyWithAWithdrawableBalance() {
        let funded = model(available: 10, reserved: 0)
        let leveraged = model(available: 50, reserved: 50, withdrawable: 0)
        let underwater = model(available: 0, reserved: 706, withdrawable: 305)
        let empty = model(available: 0, reserved: 0)

        #expect(funded.buttons.first { $0.type == .withdraw }?.isEnabled == true)
        #expect(leveraged.buttons.first { $0.type == .withdraw }?.isEnabled == false)
        #expect(underwater.buttons.first { $0.type == .withdraw }?.isEnabled == true)
        #expect(empty.buttons.first { $0.type == .withdraw }?.isEnabled == false)
        #expect(empty.buttons.first { $0.type == .deposit }?.isEnabled == true)
    }

    @Test
    func aWatchWalletIsMarkedAsOne() {
        #expect(model(available: 0, reserved: 0, walletType: .view).isWatchWallet)
        #expect(model(available: 0, reserved: 0).isWatchWallet == false)
    }
}
