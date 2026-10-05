import Components
import Foundation
import struct Gemstone.GemWalletRow
import struct Gemstone.GemWalletSection
import protocol Gemstone.GemWalletServiceProtocol
import func Gemstone.walletSections
import GemstonePrimitives
import GemstoneServices
import Localization
import Primitives
import PrimitivesComponents
import Store
import SwiftUI

@Observable
@MainActor
public final class WalletsSceneViewModel {
    private let service: any GemWalletServiceProtocol
    private let biometry: any BiometryAuthenticatable
    private let isPresentingCreateWalletSheet: Binding<Bool>
    private let isPresentingImportWalletSheet: Binding<Bool>
    private let navigationPath: Binding<NavigationPath>

    var isPresentingAlertMessage: AlertMessage?
    var walletDelete: GemWalletRow?
    private(set) var sections: [GemWalletSection] = []

    var currentWalletId: WalletId? {
        try? service.currentWalletId()
    }

    let walletsQuery: ObservableQuery<WalletListItemsQuery>

    var hasWallets: Bool { walletsQuery.value.isNotEmpty }

    public init(
        navigationPath: Binding<NavigationPath>,
        walletService: any GemWalletServiceProtocol,
        biometry: any BiometryAuthenticatable,
        isPresentingCreateWalletSheet: Binding<Bool>,
        isPresentingImportWalletSheet: Binding<Bool>,
    ) {
        self.navigationPath = navigationPath
        service = walletService
        self.biometry = biometry
        isPresentingAlertMessage = nil
        walletDelete = nil
        self.isPresentingCreateWalletSheet = isPresentingCreateWalletSheet
        self.isPresentingImportWalletSheet = isPresentingImportWalletSheet
        walletsQuery = ObservableQuery(WalletListItemsQuery(), initialValue: [])
    }

    var title: String {
        Localized.Wallets.title
    }

    var walletDeletePrompt: String {
        walletDelete?.deletePrompt.text ?? ""
    }
}

// MARK: - Business Logic

extension WalletsSceneViewModel {
    func updateSections() {
        sections = walletSections(wallets: walletsQuery.value.map { $0.toGem() }, currentWalletId: currentWalletId)
    }

    func setCurrent(_ walletId: WalletId) {
        do {
            try service.setCurrentWalletId(walletId: walletId)
        } catch {
            isPresentingAlertMessage = AlertMessage(error: error)
        }
    }
}

// MARK: - Actions

extension WalletsSceneViewModel {
    func onSelectCreateWallet() {
        isPresentingCreateWalletSheet.wrappedValue.toggle()
    }

    func onSelectImportWallet() {
        isPresentingImportWalletSheet.wrappedValue.toggle()
    }

    func onSelect(row: GemWalletRow, dismiss: DismissAction) {
        setCurrent(row.id)
        dismiss()
    }

    func onChangeWallets(dismiss: DismissAction) {
        guard !hasWallets else { return }
        dismiss()
    }

    func onEdit(row: GemWalletRow) async {
        do {
            let wallet = try await service.wallet(walletId: row.id)
            navigationPath.wrappedValue.append(Scenes.WalletDetail(wallet: wallet.toPrimitives()))
        } catch {
            isPresentingAlertMessage = AlertMessage(error: error)
        }
    }

    func onDelete(row: GemWalletRow) {
        walletDelete = row
    }

    func onPin(row: GemWalletRow) async {
        do {
            try await service.setPinned(walletId: row.id, pinned: !row.isPinned)
        } catch {
            isPresentingAlertMessage = AlertMessage(error: error)
        }
    }

    func onDeleteConfirmed(row: GemWalletRow) async {
        do {
            guard try await biometry.authenticateIfRequired(reason: Localized.Settings.Security.authentication) else {
                return
            }
            _ = try await service.deleteWallet(walletId: row.id)
        } catch {
            isPresentingAlertMessage = AlertMessage(error: error)
        }
    }
}
