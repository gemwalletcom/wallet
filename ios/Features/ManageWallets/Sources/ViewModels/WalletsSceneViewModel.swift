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

    let pinnedWalletsQuery: ObservableQuery<WalletsRequest>
    let walletsQuery: ObservableQuery<WalletsRequest>

    var pinnedWallets: [Wallet] { sorted(pinnedWalletsQuery.value) }
    var wallets: [Wallet] { sorted(walletsQuery.value) }
    var hasWallets: Bool { wallets.isNotEmpty || pinnedWallets.isNotEmpty }

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
        pinnedWalletsQuery = ObservableQuery(WalletsRequest(isPinned: true), initialValue: [])
        walletsQuery = ObservableQuery(WalletsRequest(isPinned: false), initialValue: [])
    }

    var title: String {
        Localized.Wallets.title
    }

    private func sorted(_ wallets: [Wallet]) -> [Wallet] {
        service.sorted(wallets: wallets)
    }

    var pinnedItems: [(wallet: Wallet, listItem: ListItemModel)] {
        items(pinnedWallets)
    }

    var walletItems: [(wallet: Wallet, listItem: ListItemModel)] {
        items(wallets)
    }

    private func items(_ wallets: [Wallet]) -> [(wallet: Wallet, listItem: ListItemModel)] {
        zip(wallets, walletRows(wallets: wallets.map { $0.toGem() })).map { wallet, row in
            (wallet, row.listItem)
        }
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
