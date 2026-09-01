import Components
import protocol Gemstone.GemExplorerServiceProtocol
import GemstonePrimitives
import Localization
import Onboarding
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI
import protocol Gemstone.GemWalletServiceProtocol
import Preferences
import GemstoneServices

@Observable
@MainActor
public final class WalletDetailViewModel {
    private let navigationPath: Binding<NavigationPath>
    private let walletService: any GemWalletServiceProtocol
    private let keystore: any Keystore
    private let preferences: ObservablePreferences
    private let explorerService: any GemExplorerServiceProtocol

    var nameInput: String
    var isPresentingAlertMessage: AlertMessage?
    var isPresentingDeleteConfirmation: Bool?
    var isPresentingExportWallet: ExportWalletType?

    public let walletQuery: ObservableQuery<WalletRequest>
    public var wallet: Wallet {
        walletQuery.value
    }

    public init(
        navigationPath: Binding<NavigationPath>,
        wallet: Wallet,
        walletService: any GemWalletServiceProtocol,
        keystore: any Keystore,
        preferences: ObservablePreferences,
        explorerService: any GemExplorerServiceProtocol,
    ) {
        self.navigationPath = navigationPath
        self.walletService = walletService
        self.keystore = keystore
        self.preferences = preferences
        self.explorerService = explorerService
        nameInput = wallet.name
        isPresentingAlertMessage = nil
        isPresentingDeleteConfirmation = nil
        isPresentingExportWallet = nil
        walletQuery = ObservableQuery(WalletRequest(walletId: wallet.id), initialValue: wallet)
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
        BlockExplorerLink(explorerService.getAddressUrl(chain: account.chain.rawValue, address: account.address))
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
    func rename(name: String) async throws {
        try await walletService.rename(walletId: wallet.id, newName: name)
    }

    func getMnemonicWords() async throws -> [String] {
        try await keystore.getMnemonic(wallet: wallet)
    }

    func getPrivateKey() async throws -> String {
        let chain = wallet.accounts[0].chain
        return try await keystore.getPrivateKeyEncoded(wallet: wallet, chain: chain)
    }

    func delete() async throws {
        switch try await walletService.delete(wallet) {
        case .walletsRemaining: break
        case .lastWalletDeleted: preferences.reload()
        }
    }

    func onSelectImage() {
        navigationPath.wrappedValue.append(Scenes.WalletSelectImage(wallet: wallet))
    }
}

// MARK: - Actions

extension WalletDetailViewModel {
    func onChangeWalletName() async {
        do {
            try await rename(name: nameInput)
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
