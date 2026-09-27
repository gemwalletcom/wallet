// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
import GemstonePrimitives
import GemstoneServices
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
@testable import Store
import StoreTestKit
import Testing
@testable import Wallets
import WalletsTestKit

@MainActor
struct WalletDetailSceneViewModelTests {
    @Test
    func theRowAndTheNameComeFromCore() {
        let wallet = Primitives.Wallet.mock(name: "Main Wallet")
        let model = WalletDetailSceneViewModel.mock(wallet: wallet)

        #expect(model.name == "Main Wallet")
        #expect(model.nameInput == "Main Wallet")
        #expect(model.details.row.name == "Main Wallet")
    }

    @Test
    func aMulticoinWalletHasNoSingleAddressRow() {
        let model = WalletDetailSceneViewModel.mock(wallet: .mock(type: .multicoin, accounts: [.mock(chain: .bitcoin), .mock(chain: .ethereum)]))

        #expect(model.details.address == nil)
    }

    @Test
    func aSingleChainWalletShowsItsAddressWithAnExplorerLink() throws {
        let account = Account.mock(chain: .ethereum, address: "0xabc")
        let model = WalletDetailSceneViewModel.mock(wallet: .mock(type: .single, accounts: [account]))

        let address = try #require(model.details.address)
        #expect(address.address == "0xabc")
        #expect(address.menu.contains { if case let .open(_, url) = $0 { url.contains("0xabc") } else { false } })
    }

    @Test
    func renamingAWalletThatIsGoneShowsTheError() async {
        let model = WalletDetailSceneViewModel.mock(wallet: .mock(id: .multicoin(address: "0xmissing")))
        model.nameInput = "Renamed"

        await model.onChangeWalletName()

        #expect(model.isPresentingAlertMessage != nil)
    }

    @Test
    func renamingStoresTheNewName() async throws {
        let wallet = Primitives.Wallet.mock(id: .multicoin(address: "0x1"), name: "Old")
        let db = DB.mock(wallets: [wallet])
        let walletStore = WalletStore.mock(db: db)
        let model = WalletDetailSceneViewModel.mock(wallet: wallet, service: GemWalletService.mock(db: db))
        model.nameInput = "New"

        await model.onChangeWalletName()

        #expect(model.isPresentingAlertMessage == nil)
        #expect(try walletStore.getWallets().first?.name == "New")
    }

    @Test
    func askingToDeleteOpensTheConfirmation() {
        let model = WalletDetailSceneViewModel.mock()

        model.onSelectDelete()

        #expect(model.isPresentingDeleteConfirmation == true)
    }

    @Test
    func deletingTheOnlyWalletSucceeds() async throws {
        let wallet = Primitives.Wallet.mock(id: .multicoin(address: "0x1"))
        let db = DB.mock(wallets: [wallet])
        let walletStore = WalletStore.mock(db: db)
        let biometry = BiometryAuthenticationMock(requiresAuthentication: false)
        let model = WalletDetailSceneViewModel.mock(wallet: wallet, service: GemWalletService.mock(db: db), biometry: biometry)

        #expect(await model.onDelete())
        #expect(biometry.authenticateCallsCount == 0)
        #expect(try walletStore.getWallets().isEmpty)
    }

    @Test
    func deletingAuthenticatesWhenAuthenticationIsEnabled() async throws {
        let wallet = Primitives.Wallet.mock(id: .multicoin(address: "0x1"))
        let db = DB.mock(wallets: [wallet])
        let walletStore = WalletStore.mock(db: db)
        let biometry = BiometryAuthenticationMock()
        let model = WalletDetailSceneViewModel.mock(wallet: wallet, service: GemWalletService.mock(db: db), biometry: biometry)

        #expect(await model.onDelete())
        #expect(biometry.authenticateCallsCount == 1)
        #expect(try walletStore.getWallets().isEmpty)
    }

    @Test
    func cancellingAuthenticationKeepsTheWallet() async throws {
        let wallet = Primitives.Wallet.mock(id: .multicoin(address: "0x1"))
        let db = DB.mock(wallets: [wallet])
        let walletStore = WalletStore.mock(db: db)
        let biometry = BiometryAuthenticationMock()
        biometry.authenticateError = BiometryAuthenticationError.cancelledByUser
        let model = WalletDetailSceneViewModel.mock(wallet: wallet, service: GemWalletService.mock(db: db), biometry: biometry)

        #expect(await model.onDelete() == false)
        #expect(model.isPresentingAlertMessage == nil)
        #expect(try walletStore.getWallets().count == 1)
    }

    @Test
    func aFailedAuthenticationKeepsTheWallet() async throws {
        let wallet = Primitives.Wallet.mock(id: .multicoin(address: "0x1"))
        let db = DB.mock(wallets: [wallet])
        let walletStore = WalletStore.mock(db: db)
        let biometry = BiometryAuthenticationMock()
        biometry.authenticateError = BiometryAuthenticationError.authenticationFailed
        let model = WalletDetailSceneViewModel.mock(wallet: wallet, service: GemWalletService.mock(db: db), biometry: biometry)

        #expect(await model.onDelete() == false)
        #expect(model.isPresentingAlertMessage == nil)
        #expect(try walletStore.getWallets().count == 1)
    }

    @Test
    func exportingASecretAWatchWalletDoesNotHaveShowsTheError() async {
        let model = WalletDetailSceneViewModel.mock(wallet: .mock(id: .multicoin(address: "0xmissing")))

        await model.onShowSecret()

        #expect(model.isPresentingExportWallet == nil)
        #expect(model.isPresentingAlertMessage != nil)
    }
}
