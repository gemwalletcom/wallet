import Components
import struct Gemstone.GemAddressRow
import enum Gemstone.GemServiceError
import struct Gemstone.GemWalletDetails
import struct Gemstone.GemWalletRow
import enum Gemstone.GemWalletSecret
import enum Gemstone.GemWalletSecretKind
import protocol Gemstone.GemWalletServiceProtocol
import GemstonePrimitives
import GemstoneServices
import Localization
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI

@Observable
@MainActor
public final class WalletDetailSceneViewModel {
    private let navigationPath: Binding<NavigationPath>
    private let service: any GemWalletServiceProtocol
    private let biometry: any BiometryAuthenticatable

    var nameInput: String
    var isPresentingAlertMessage: AlertMessage?
    var isPresentingDeleteConfirmation: Bool?
    var isPresentingExportWallet: GemWalletSecret?

    public let walletQuery: ObservableQuery<MappedQuery<WalletQuery, WalletDetailsValue>>
    public var wallet: Wallet {
        walletQuery.value.wallet
    }

    public init(
        navigationPath: Binding<NavigationPath>,
        wallet: Wallet,
        service: any GemWalletServiceProtocol,
        biometry: any BiometryAuthenticatable,
    ) {
        self.navigationPath = navigationPath
        self.service = service
        self.biometry = biometry
        nameInput = wallet.name
        isPresentingAlertMessage = nil
        isPresentingDeleteConfirmation = nil
        isPresentingExportWallet = nil
        let details: @Sendable (Wallet) -> WalletDetailsValue = { [service] in
            WalletDetailsValue(wallet: $0, details: service.walletDetails(wallet: $0.toGem()))
        }
        walletQuery = ObservableQuery(MappedQuery(WalletQuery(walletId: wallet.id), transform: details), initialValue: details(wallet))
    }

    var details: GemWalletDetails {
        walletQuery.value.details
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

    func showSecretListItem(for secretKind: GemWalletSecretKind) -> ListItemModel {
        ListItemModel(title: Localized.Common.show(secretKind.title))
    }

    var secretKind: GemWalletSecretKind? {
        details.secretKind
    }

    var addressRow: GemAddressRow? {
        details.address
    }

    var avatarAssetImage: AssetImage {
        let avatar = row.avatarImage
        return AssetImage(
            type: avatar.type,
            imageURL: avatar.imageURL,
            placeholder: avatar.placeholder,
            chainPlaceholder: Images.Wallets.editFilled,
        )
    }
}

// MARK: - Business Logic

extension WalletDetailSceneViewModel {
    func rename(name: String) async throws {
        try await service.rename(walletId: wallet.id, newName: name)
    }

    func delete() async throws {
        _ = try await service.delete(wallet)
    }

    func onSelectImage() {
        navigationPath.wrappedValue.append(Scenes.WalletImage(wallet: wallet))
    }
}

// MARK: - Actions

extension WalletDetailSceneViewModel {
    func onChangeWalletName() async {
        do {
            try await rename(name: nameInput)
        } catch let error as GemServiceError {
            isPresentingAlertMessage = AlertMessage(message: error.text().text)
        } catch {
            debugLog("wallet detail error: \(error)")
        }
    }

    func onShowSecret() async {
        do {
            isPresentingExportWallet = try await service.exportSecret(walletId: wallet.id.id)
        } catch {
            isPresentingAlertMessage = AlertMessage(error: error)
        }
    }

    func onSelectDelete() {
        isPresentingDeleteConfirmation = true
    }

    func onDelete() async -> Bool {
        do {
            guard try await biometry.authenticateIfRequired(reason: Localized.Settings.Security.authentication) else {
                return false
            }
            try await delete()
            return true
        } catch let error as GemServiceError {
            isPresentingAlertMessage = AlertMessage(message: error.text().text)
            return false
        } catch {
            debugLog("wallet detail error: \(error)")
            return false
        }
    }
}

public struct WalletDetailsValue: Equatable, Sendable {
    let wallet: Wallet
    let details: GemWalletDetails
}
