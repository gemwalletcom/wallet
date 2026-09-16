// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
import GemstonePrimitives
import GemstoneServices
import GemstoneServicesTestKit
@testable import ManageWallets
import Primitives
import PrimitivesTestKit
@testable import Store
import StoreTestKit
import SwiftUI
import Testing

@MainActor
struct WalletDetailViewModelTests {
    private func model(
        wallet: Primitives.Wallet,
        db: DB = .mock(),
        service: GemWalletService? = nil,
    ) -> WalletDetailViewModel {
        WalletDetailViewModel(
            navigationPath: .constant(NavigationPath()),
            wallet: wallet,
            service: service ?? GemWalletService.mock(db: db),
            preferences: .mock(),
        )
    }

    @Test
    func theRowAndTheNameComeFromCore() {
        let wallet = Primitives.Wallet.mock(name: "Main Wallet")
        let model = model(wallet: wallet)

        #expect(model.name == "Main Wallet")
        #expect(model.nameInput == "Main Wallet")
        #expect(model.row.name == "Main Wallet")
    }

    @Test
    func aMulticoinWalletHasNoSingleAddressRow() {
        let model = model(wallet: .mock(type: .multicoin, accounts: [.mock(chain: .bitcoin), .mock(chain: .ethereum)]))

        #expect(model.address == nil)
    }

    @Test
    func aSingleChainWalletShowsItsAddressWithAnExplorerLink() throws {
        let account = Account.mock(chain: .ethereum, address: "0xabc")
        let model = model(wallet: .mock(type: .single, accounts: [account]))

        let address = try #require(model.address)
        guard case let .account(simple, link) = address else {
            Issue.record("expected an account address, got \(address)")
            return
        }
        #expect(simple.address == "0xabc")
        #expect(link.url.absoluteString.contains("0xabc"))
    }

    @Test
    func renamingAWalletThatIsGoneShowsTheError() async {
        let model = model(wallet: .mock(id: .multicoin(address: "0xmissing")))
        model.nameInput = "Renamed"

        await model.onChangeWalletName()

        #expect(model.isPresentingAlertMessage != nil)
    }

    @Test
    func renamingStoresTheNewName() async throws {
        let db = DB.mock()
        let walletStore = WalletStore.mock(db: db)
        let wallet = Primitives.Wallet.mock(id: .multicoin(address: "0x1"), name: "Old")
        try walletStore.addWallet(wallet)
        let model = model(wallet: wallet, db: db)
        model.nameInput = "New"

        await model.onChangeWalletName()

        #expect(model.isPresentingAlertMessage == nil)
        #expect(try walletStore.getWallets().first?.name == "New")
    }

    @Test
    func askingToDeleteOpensTheConfirmation() {
        let model = model(wallet: .mock())

        model.onSelectDelete()

        #expect(model.isPresentingDeleteConfirmation == true)
    }

    @Test
    func deletingTheOnlyWalletSucceeds() async throws {
        let db = DB.mock()
        let walletStore = WalletStore.mock(db: db)
        let wallet = Primitives.Wallet.mock(id: .multicoin(address: "0x1"))
        try walletStore.addWallet(wallet)
        let model = model(wallet: wallet, db: db)

        #expect(await model.onDelete())
        #expect(try walletStore.getWallets().isEmpty)
    }

    @Test
    func exportingASecretAWatchWalletDoesNotHaveShowsTheError() async {
        let model = model(wallet: .mock(id: .multicoin(address: "0xmissing")))

        model.onShowSecret()
        await settle { model.isPresentingAlertMessage != nil }

        #expect(model.isPresentingExportWallet == nil)
        #expect(model.isPresentingAlertMessage != nil)
    }

    private func settle(until condition: () -> Bool) async {
        for _ in 0 ..< 200 {
            await Task.yield()
            if condition() { return }
            try? await Task.sleep(for: .milliseconds(5))
        }
    }
}
