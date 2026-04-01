import Components
import ExplorerService
import Localization
import Onboarding
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI
import WalletService

@Observable
@MainActor
public final class WalletDetailViewModel {
    private let navigationPath: Binding<NavigationPath>
    let wallet: Wallet
    let walletService: WalletService
    private let explorerService: any ExplorerLinkFetchable

    var nameInput: String
    var isPresentingAlertMessage: AlertMessage?
    var isPresentingDeleteConfirmation: Bool?
    var isPresentingExportWallet: ExportWalletType?

    public let walletQuery: ObservableQuery<WalletRequest>
    public var dbWallet: Wallet? { walletQuery.value }

    public init(
        navigationPath: Binding<NavigationPath>,
        wallet: Wallet,
        walletService: WalletService,
        explorerService: any ExplorerLinkFetchable = ExplorerService.standard,
    ) {
        self.navigationPath = navigationPath
        self.wallet = wallet
        self.walletService = walletService
        self.explorerService = explorerService
        nameInput = wallet.name
        isPresentingAlertMessage = nil
        isPresentingDeleteConfirmation = nil
        isPresentingExportWallet = nil
        walletQuery = ObservableQuery(WalletRequest(walletId: wallet.walletId), initialValue: wallet)
    }

    var name: String {
        wallet.name
    }

    var title: String {
        Localized.Common.wallet
    }

    var address: WalletDetailAddress? {
        switch wallet.type {
        case .multicoin:
            return .none
        case .single, .view, .privateKey:
            guard let account = wallet.accounts.first else { return .none }
            return WalletDetailAddress.account(
                SimpleAccount(
                    name: .none,
                    chain: account.chain,
                    address: account.address,
                    assetImage: .none,
                ),
            )
        }
    }

    func addressLink(account: SimpleAccount) -> BlockExplorerLink {
        explorerService.addressUrl(chain: account.chain, address: account.address)
    }

    func avatarAssetImage(for wallet: Wallet) -> AssetImage {
        let avatar = WalletViewModel(wallet: wallet).avatarImage
        return AssetImage(
            type: avatar.type,
            imageURL: avatar.imageURL,
            placeholder: avatar.placeholder,
            chainPlaceholder: Images.Wallets.editFilled,
        )
    }
}

// MARK: - Business Logic

extension WalletDetailViewModel {
    func rename(name: String) throws {
        try walletService.rename(walletId: wallet.walletId, newName: name)
    }

    func getMnemonicWords() async throws -> [String] {
        try await walletService.getMnemonic(wallet: wallet)
    }

    func getPrivateKey() async throws -> String {
        let chain = wallet.accounts[0].chain
        return try await walletService.getPrivateKeyEncoded(wallet: wallet, chain: chain)
    }

    func delete() async throws {
        try await walletService.delete(wallet)
    }

    func onSelectImage() {
        navigationPath.wrappedValue.append(Scenes.WalletSelectImage(wallet: wallet))
    }
}

// MARK: - Actions

extension WalletDetailViewModel {
    func onChangeWalletName() {
        do {
            try rename(name: nameInput)
        } catch {
            isPresentingAlertMessage = AlertMessage(message: error.localizedDescription)
        }
    }

    func onShowSecretPhrase() {
        Task {
            do {
                isPresentingExportWallet = try await .words(getMnemonicWords())
            } catch {
                isPresentingAlertMessage = AlertMessage(message: error.localizedDescription)
            }
        }
    }

    func onShowPrivateKey() {
        Task {
            do {
                isPresentingExportWallet = try await .privateKey(getPrivateKey())
            } catch {
                isPresentingAlertMessage = AlertMessage(message: error.localizedDescription)
            }
        }
    }

    func onSelectDelete() {
        isPresentingDeleteConfirmation = true
    }

    func onDelete() async -> Bool {
        do {
            try await delete()
            return true
        } catch {
            isPresentingAlertMessage = AlertMessage(message: error.localizedDescription)
            return false
        }
    }
}
