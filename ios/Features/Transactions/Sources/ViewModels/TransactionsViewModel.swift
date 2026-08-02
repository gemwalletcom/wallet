// Copyright (c). Gem Wallet. All rights reserved.

import Components
import ExplorerService
import Foundation
import Localization
import Preferences
import Primitives
import PrimitivesComponents
import Store
import TransactionsService
import WalletService

@Observable
@MainActor
public final class TransactionsViewModel {
    public let explorerService: any ExplorerLinkFetchable = ExplorerService.standard
    public let transactionsService: TransactionsService
    public let preferences: ObservablePreferences

    private let walletService: WalletService
    private let type: TransactionsRequestType

    public private(set) var wallet: Wallet

    public var transactions: [TransactionExtended] {
        filterModel.query.value
    }

    public var filterModel: TransactionsFilterViewModel

    public var isPresentingSheet: TransactionsSheetType?
    public var isPresentingToastMessage: ToastMessage?

    public init(
        transactionsService: TransactionsService,
        walletService: WalletService,
        wallet: Wallet,
        type: TransactionsRequestType,
        preferences: ObservablePreferences = .default,
    ) {
        self.walletService = walletService
        self.transactionsService = transactionsService

        self.type = type
        self.wallet = wallet
        filterModel = TransactionsFilterViewModel(wallet: wallet, type: type)
        self.preferences = preferences
    }

    public var title: String {
        Localized.Activity.title
    }

    public var currentWallet: Wallet? {
        walletService.currentWallet
    }

    public var walletId: WalletId {
        wallet.id
    }

    public var currency: String {
        preferences.preferences.currency
    }

    public var hideBalance: Bool {
        preferences.isHideBalanceEnabled
    }

    public var emptyContentModel: EmptyContentTypeViewModel {
        if !filterModel.isAnyFilterSpecified {
            EmptyContentTypeViewModel(type: .activity(receive: onSelectReceive, buy: onSelectBuy, isViewOnly: wallet.isViewOnly))
        } else {
            EmptyContentTypeViewModel(type: .search(type: .activity, action: onSelectCleanFilters))
        }
    }
}

// MARK: - Business Logic

public extension TransactionsViewModel {
    func onChangeWallet(_: Wallet?, _ newWallet: Wallet?) {
        if let newWallet, wallet != newWallet {
            refresh(for: newWallet)
        }
    }

    func onSelectFilterButton() {
        isPresentingSheet = .filter
    }

    func fetch() async {
        do {
            try await transactionsService.updateAll(walletId: walletId)
        } catch {
            debugLog("fetch getTransactions error \(error)")
        }
    }
}

// MARK: - Private

extension TransactionsViewModel {
    private func refresh(for wallet: Wallet) {
        self.wallet = wallet
        filterModel = TransactionsFilterViewModel(wallet: wallet, type: type)
    }

    private func onSelectCleanFilters() {
        refresh(for: wallet)
    }

    private func onSelectReceive() {
        isPresentingSheet = .selectAsset(.receive(.asset))
    }

    private func onSelectBuy() {
        isPresentingSheet = .selectAsset(.buy)
    }
}
