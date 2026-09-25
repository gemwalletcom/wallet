import Components
import Foundation
import protocol Gemstone.GemWalletServiceProtocol
import func Gemstone.walletRows
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
    var walletDelete: Wallet?

    var currentWalletId: WalletId? {
        service.currentWalletId
    }

    let pinnedWalletsQuery: ObservableQuery<MappedQuery<WalletsQuery, [WalletEntry]>>
    let walletsQuery: ObservableQuery<MappedQuery<WalletsQuery, [WalletEntry]>>

    var hasWallets: Bool { walletsQuery.value.isNotEmpty || pinnedWalletsQuery.value.isNotEmpty }

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
        let entries: @Sendable ([Wallet]) -> [WalletEntry] = { [walletService] wallets in
            let sorted = walletService.sorted(wallets: wallets)
            return zip(sorted, walletRows(wallets: sorted.map { $0.toGem() })).map(WalletEntry.init)
        }
        pinnedWalletsQuery = ObservableQuery(MappedQuery(WalletsQuery(isPinned: true), transform: entries), initialValue: [])
        walletsQuery = ObservableQuery(MappedQuery(WalletsQuery(isPinned: false), transform: entries), initialValue: [])
    }

    var title: String {
        Localized.Wallets.title
    }

    var pinnedItems: [(wallet: Wallet, listItem: ListItemModel)] {
        pinnedWalletsQuery.value.map { ($0.wallet, $0.row.listItem) }
    }

    var walletItems: [(wallet: Wallet, listItem: ListItemModel)] {
        walletsQuery.value.map { ($0.wallet, $0.row.listItem) }
    }
}

// MARK: - Business Logic

extension WalletsSceneViewModel {
    func setCurrent(_ walletId: WalletId) {
        do {
            try service.setCurrentWalletId(walletId: walletId.id)
        } catch {
            isPresentingAlertMessage = AlertMessage(error: error)
        }
    }

    func onEdit(wallet: Wallet) {
        navigationPath.wrappedValue.append(Scenes.WalletDetail(wallet: wallet))
    }

    private func delete(_ wallet: Wallet) async throws {
        _ = try await service.delete(wallet)
    }

    private func pin(_ wallet: Wallet) async throws {
        if wallet.isPinned {
            try await service.unpin(wallet: wallet)
        } else {
            try await service.pin(wallet: wallet)
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

    func onSelect(wallet: Wallet, dismiss: DismissAction) {
        setCurrent(wallet.id)
        dismiss()
    }

    func onChangeWallets(dismiss: DismissAction) {
        guard !hasWallets else { return }
        dismiss()
    }

    func onDelete(wallet: Wallet) {
        walletDelete = wallet
    }

    func onPin(wallet: Wallet) async {
        do {
            try await pin(wallet)
        } catch {
            isPresentingAlertMessage = AlertMessage(error: error)
        }
    }

    func onDeleteConfirmed(wallet: Wallet) async {
        do {
            guard try await biometry.authenticateIfRequired(reason: Localized.Settings.Security.authentication) else {
                return
            }
            try await delete(wallet)
        } catch {
            isPresentingAlertMessage = AlertMessage(error: error)
        }
    }
}
