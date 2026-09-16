// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
import GemstonePrimitives
import GemstoneServices
import GemstoneServicesTestKit
@testable import ManageWallets
import ManageWalletsTestKit
import Primitives
import PrimitivesTestKit
@testable import Store
import StoreTestKit
import Testing

@MainActor
struct WalletDetailViewModelTests {
    @Test
    func theRowAndTheNameComeFromCore() {
        let wallet = Primitives.Wallet.mock(name: "Main Wallet")
        let model = WalletDetailViewModel.mock(wallet: wallet)

        #expect(model.name == "Main Wallet")
        #expect(model.nameInput == "Main Wallet")
        #expect(model.row.name == "Main Wallet")
    }

    @Test
    func aMulticoinWalletHasNoSingleAddressRow() {
        let model = WalletDetailViewModel.mock(wallet: .mock(type: .multicoin, accounts: [.mock(chain: .bitcoin), .mock(chain: .ethereum)]))

        #expect(model.address == nil)
    }

    @Test
    func aSingleChainWalletShowsItsAddressWithAnExplorerLink() throws {
        let account = Account.mock(chain: .ethereum, address: "0xabc")
        let model = WalletDetailViewModel.mock(wallet: .mock(type: .single, accounts: [account]))

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
        let model = WalletDetailViewModel.mock(wallet: .mock(id: .multicoin(address: "0xmissing")))
        model.nameInput = "Renamed"

        await model.onChangeWalletName()

        #expect(model.isPresentingAlertMessage != nil)
    }

    @Test
    func renamingStoresTheNewName() async throws {
        let wallet = Primitives.Wallet.mock(id: .multicoin(address: "0x1"), name: "Old")
        let db = try DB.mockWithWallets([wallet])
        let walletStore = WalletStore.mock(db: db)
        let model = WalletDetailViewModel.mock(wallet: wallet, service: GemWalletService.mock(db: db))
        model.nameInput = "New"

        await model.onChangeWalletName()

        #expect(model.isPresentingAlertMessage == nil)
        #expect(try walletStore.getWallets().first?.name == "New")
    }

    @Test
    func askingToDeleteOpensTheConfirmation() {
        let model = WalletDetailViewModel.mock()

        model.onSelectDelete()

        #expect(model.isPresentingDeleteConfirmation == true)
    }

    @Test
    func deletingTheOnlyWalletSucceeds() async throws {
        let wallet = Primitives.Wallet.mock(id: .multicoin(address: "0x1"))
        let db = try DB.mockWithWallets([wallet])
        let walletStore = WalletStore.mock(db: db)
        let model = WalletDetailViewModel.mock(wallet: wallet, service: GemWalletService.mock(db: db))

        #expect(await model.onDelete())
        #expect(try walletStore.getWallets().isEmpty)
    }

    @Test
    func exportingASecretAWatchWalletDoesNotHaveShowsTheError() async {
        let model = WalletDetailViewModel.mock(wallet: .mock(id: .multicoin(address: "0xmissing")))

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
