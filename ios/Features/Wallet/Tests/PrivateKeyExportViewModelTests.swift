// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
import GemstonePrimitives
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
@testable import Store
import StoreTestKit
import Testing
@testable import Wallet

@MainActor
struct PrivateKeyExportViewModelTests {
    @Test
    func continuingAMulticoinExportOpensTheAccountList() async {
        let model = PrivateKeyExportViewModel(walletId: .multicoin(address: "0x1"), export: .privateKeyAccounts, service: GemWalletService.mock())

        await model.onContinue()

        #expect(model.navigationPath.count == 1)
        #expect(model.isPresentingAlertMessage == nil)
    }

    @Test
    func theAccountListHasOnlyChainsWithAnExportableKey() async {
        let wallet = Primitives.Wallet.mock(id: .multicoin(address: "0x1"), type: .multicoin, accounts: [.mock(chain: .bitcoin), .mock(chain: .ethereum), .mock(chain: .solana)])
        let model = PrivateKeyExportViewModel(walletId: wallet.id, export: .privateKeyAccounts, service: GemWalletService.mock(db: DB.mock(wallets: [wallet])))

        await model.fetchAccounts()

        #expect(model.accounts.map(\.row.chain) == [Primitives.Chain.ethereum.rawValue, Primitives.Chain.solana.rawValue])
    }

    @Test
    func aKeyThatCannotBeExportedShowsTheErrorAndRevealsNothing() async {
        let model = PrivateKeyExportViewModel(walletId: .multicoin(address: "0xmissing"), export: .privateKey(chain: Primitives.Chain.ethereum.rawValue), service: GemWalletService.mock())

        await model.onContinue()

        #expect(model.navigationPath.isEmpty)
        #expect(model.isPresentingAlertMessage != nil)
    }
}
