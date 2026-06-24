import Components
import Foundation
import Localization
import Preferences
import Primitives
import Store
import SwiftUI
import WalletService

@Observable
@MainActor
public final class WalletsSceneViewModel {
    #if DEBUG
        public static let walletsLimit = 1000
    #else
        public static let walletsLimit = 100
    #endif

    private let service: WalletService
    private let isPresentingCreateWalletSheet: Binding<Bool>
    private let isPresentingImportWalletSheet: Binding<Bool>
    private let navigationPath: Binding<NavigationPath>

    var isPresentingAlertMessage: AlertMessage?
    var walletDelete: Wallet?
    var currentWalletId: WalletId?

    let pinnedWalletsQuery: ObservableQuery<WalletsRequest>
    let walletsQuery: ObservableQuery<WalletsRequest>

    var pinnedWallets: [Wallet] {
        pinnedWalletsQuery.value
    }

    var wallets: [Wallet] {
        walletsQuery.value
    }

    public init(
        navigationPath: Binding<NavigationPath>,
        walletService: WalletService,
        isPresentingCreateWalletSheet: Binding<Bool>,
        isPresentingImportWalletSheet: Binding<Bool>,
    ) {
        self.navigationPath = navigationPath
        service = walletService
        currentWalletId = service.currentWalletId
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
}

// MARK: - Business Logic

extension WalletsSceneViewModel {
    func setCurrent(_ walletId: WalletId) {
        service.setCurrent(for: walletId)
        currentWalletId = walletId
    }

    func onEdit(wallet: Wallet) {
        navigationPath.wrappedValue.append(Scenes.WalletDetail(wallet: wallet))
    }

    private func delete(_ wallet: Wallet) async throws {
        try await service.delete(wallet)
    }

    private func pin(_ wallet: Wallet) throws {
        if wallet.isPinned {
            try service.unpin(wallet: wallet)
        } else {
            try service.pin(wallet: wallet)
        }
    }
}

// MARK: - Actions

extension WalletsSceneViewModel {
    func onSelectCreateWallet() {
        guard validate() else {
            return
        }
        isPresentingCreateWalletSheet.wrappedValue.toggle()
    }

    func onSelectImportWallet() {
        guard validate() else {
            return
        }
        isPresentingImportWalletSheet.wrappedValue.toggle()
    }

    func onSelect(wallet: Wallet, dismiss: DismissAction) {
        setCurrent(wallet.id)
        dismiss()
    }

    func onDelete(wallet: Wallet) {
        walletDelete = wallet
    }

    func onPin(wallet: Wallet) {
        do {
            try pin(wallet)
        } catch {
            isPresentingAlertMessage = AlertMessage(message: error.localizedDescription)
        }
    }

    func onDeleteConfirmed(wallet: Wallet) async {
        do {
            try await delete(wallet)
            currentWalletId = service.currentWalletId
        } catch {
            isPresentingAlertMessage = AlertMessage(message: error.localizedDescription)
        }
    }
}

// MARK: - Private

extension WalletsSceneViewModel {
    private func validate() -> Bool {
        // fix: https://github.com/gemwalletcom/gem-ios/issues/1067
        if wallets.count > WalletsSceneViewModel.walletsLimit {
            isPresentingAlertMessage = AlertMessage(
                title: Localized.Errors.Wallets.Limit.title,
                message: Localized.Errors.Wallets.Limit.description(WalletsSceneViewModel.walletsLimit),
            )
            return false
        }
        return true
    }
}
