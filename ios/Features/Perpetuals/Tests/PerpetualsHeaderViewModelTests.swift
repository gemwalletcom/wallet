// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import PrimitivesComponents
@testable import Perpetuals
import Testing

struct PerpetualsHeaderViewModelTests {
    @Test
    func theHeaderShowsTheTotalAndTheAvailableBalance() {
        let model = PerpetualsHeaderViewModel(walletType: .multicoin, balance: WalletBalance(total: 1250, available: 300))

        #expect(model.title.contains("1,250"))
        #expect(model.subtitle?.contains("300") == true)
    }

    @Test
    func withdrawIsOfferedOnlyWithAnAvailableBalance() {
        let funded = PerpetualsHeaderViewModel(walletType: .multicoin, balance: WalletBalance(total: 10, available: 10))
        let empty = PerpetualsHeaderViewModel(walletType: .multicoin, balance: .zero)

        #expect(funded.buttons.first { $0.type == .withdraw }?.isEnabled == true)
        #expect(empty.buttons.first { $0.type == .withdraw }?.isEnabled == false)
        #expect(empty.buttons.first { $0.type == .deposit }?.isEnabled == true)
    }

    @Test
    func aWatchWalletIsMarkedAsOne() {
        #expect(PerpetualsHeaderViewModel(walletType: .view, balance: .zero).isWatchWallet)
        #expect(PerpetualsHeaderViewModel(walletType: .multicoin, balance: .zero).isWatchWallet == false)
    }
}
