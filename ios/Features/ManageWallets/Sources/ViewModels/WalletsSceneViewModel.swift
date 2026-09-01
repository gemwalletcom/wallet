import Components
import Foundation
import GemstonePrimitives
import Localization
import Preferences
import Primitives
import Store
import SwiftUI
import protocol Gemstone.GemWalletServiceProtocol
import protocol Gemstone.GemWalletSessionServiceProtocol

@Observable
@MainActor
public final class WalletsSceneViewModel {

    public static let walletsLimit = 100

    private let service: any GemWalletServiceProtocol
    private let session: any GemWalletSessionServiceProtocol
    private let preferences: ObservablePreferences
    private let isPresentingCreateWalletSheet: Binding<Bool>
    private let isPresentingImportWalletSheet: Binding<Bool>
    private let navigationPath: Binding<NavigationPath>

    var isPresentingAlertMessage: AlertMessage?
    var walletDelete: Wallet?

    var currentWalletId: WalletId? {
        session.currentWalletId
    }

    let pinnedWalletsQuery: ObservableQuery<WalletsRequest>
    let walletsQuery: ObservableQuery<WalletsRequest>

    var pinnedWallets: [Wallet] { sorted(pinnedWalletsQuery.value) }
    var wallets: [Wallet] { sorted(walletsQuery.value) }
    var hasWallets: Bool { wallets.isNotEmpty || pinnedWallets.isNotEmpty }

    public init(
        navigationPath: Binding<NavigationPath>,
        walletService: any GemWalletServiceProtocol,
        session: any GemWalletSessionServiceProtocol,
        preferences: ObservablePreferences,
        isPresentingCreateWalletSheet: Binding<Bool>,
        isPresentingImportWalletSheet: Binding<Bool>,
    ) {
        self.navigationPath = navigationPath
        service = walletService
        self.session = session
        self.preferences = preferences
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
}

// MARK: - Business Logic

extension WalletsSceneViewModel {
    func setCurrent(_ walletId: WalletId) {
        do {
            try session.setCurrent(walletId: walletId)
        } catch {
            debugLog("set current wallet error: \(error)")
        }
    }

    func onEdit(wallet: Wallet) {
        navigationPath.wrappedValue.append(Scenes.WalletDetail(wallet: wallet))
    }

    private func delete(_ wallet: Wallet) async throws {
        switch try await service.delete(wallet) {
        case .walletsRemaining: break
        case .lastWalletDeleted: preferences.reload()
        }
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
            isPresentingAlertMessage = AlertMessage(message: error.localizedDescription)
        }
    }

    func onDeleteConfirmed(wallet: Wallet) async {
        do {
            try await delete(wallet)
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
