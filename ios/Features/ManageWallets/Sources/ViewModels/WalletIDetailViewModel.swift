import Components
import GemstonePrimitives
import Localization
import Onboarding
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI
import struct Gemstone.GemWalletDetails
import struct Gemstone.GemWalletRow
import enum Gemstone.GemWalletSecret
import enum Gemstone.GemWalletSecretKind
import func Gemstone.walletDetails
import func Gemstone.walletRow
import protocol Gemstone.GemWalletServiceProtocol
import GemstoneServices

@Observable
@MainActor
public final class WalletDetailViewModel {
    private let navigationPath: Binding<NavigationPath>
    private let service: any GemWalletServiceProtocol
    private let preferences: ObservablePreferences

    var nameInput: String
    var isPresentingAlertMessage: AlertMessage?
    var isPresentingDeleteConfirmation: Bool?
    var isPresentingExportWallet: GemWalletSecret?

    public let walletQuery: ObservableQuery<WalletRequest>
    public var wallet: Wallet {
        walletQuery.value
    }

    public init(
        navigationPath: Binding<NavigationPath>,
        wallet: Wallet,
        service: any GemWalletServiceProtocol,
        preferences: ObservablePreferences,
    ) {
        self.navigationPath = navigationPath
        self.service = service
        self.preferences = preferences
        nameInput = wallet.name
        isPresentingAlertMessage = nil
        isPresentingDeleteConfirmation = nil
        isPresentingExportWallet = nil
        walletQuery = ObservableQuery(WalletRequest(walletId: wallet.id), initialValue: wallet)
    }

    var details: GemWalletDetails {
        walletDetails(wallet: wallet.toGem())
    }

    var row: GemWalletRow {
        details.row
    }

    var name: String {
        row.name
    }

    var title: String {
        Localized.Common.wallet
    }

    var secretKind: GemWalletSecretKind? {
        details.secretKind
    }

    var address: WalletDetailAddress? {
        guard let account = details.address?.toPrimitives() else { return .none }
        return .account(
            SimpleAccount(name: .none, chain: account.chain, address: account.address, assetImage: .none),
            link: service.addressUrl(chain: account.chain.rawValue, address: account.address).toPrimitives(),
        )
    }

    func avatarAssetImage(for wallet: Wallet) -> AssetImage {
        let avatar = walletRow(wallet: wallet.toGem()).avatarImage
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
        try await service.rename(walletId: wallet.id, newName: name)
    }

    func delete() async throws {
        preferences.reload(after: try await service.delete(wallet))
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

    func onShowSecret() {
        Task {
            do {
                isPresentingExportWallet = try await service.exportSecret(walletId: wallet.id.id)
            } catch {
                isPresentingAlertMessage = AlertMessage(error: error)
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
