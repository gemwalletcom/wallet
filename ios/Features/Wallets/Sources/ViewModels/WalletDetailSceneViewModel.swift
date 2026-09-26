import Components
import enum Gemstone.GemServiceError
import struct Gemstone.GemWalletDetails
import enum Gemstone.GemWalletSecret
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

    private let wallet: Wallet

    public let detailsQuery: ObservableQuery<MappedQuery<WalletQuery, GemWalletDetails>>

    public init(
        navigationPath: Binding<NavigationPath>,
        wallet: Wallet,
        service: any GemWalletServiceProtocol,
        biometry: any BiometryAuthenticatable,
    ) {
        self.navigationPath = navigationPath
        self.service = service
        self.biometry = biometry
        self.wallet = wallet
        nameInput = wallet.name
        isPresentingAlertMessage = nil
        isPresentingDeleteConfirmation = nil
        isPresentingExportWallet = nil
        let details: @Sendable (Wallet) -> GemWalletDetails = { [service] in
            service.walletDetails(wallet: $0.toGem())
        }
        detailsQuery = ObservableQuery(MappedQuery(WalletQuery(walletId: wallet.id), transform: details), initialValue: details(wallet))
    }

    var details: GemWalletDetails {
        detailsQuery.value
    }

    var name: String {
        details.row.name
    }

    var title: String {
        Localized.Common.wallet
    }

    var avatarAssetImage: AssetImage {
        let avatar = details.row.avatarImage
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
