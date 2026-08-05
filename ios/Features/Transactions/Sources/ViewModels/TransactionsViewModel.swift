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
import WalletSessionService

@Observable
@MainActor
public final class TransactionsViewModel {
    public let explorerService: any ExplorerLinkFetchable = ExplorerService.standard
    public let transactionsService: TransactionsService
    public let preferences: Preferences

    private let walletSessionService: any WalletSessionManageable
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
        walletSessionService: any WalletSessionManageable,
        wallet: Wallet,
        type: TransactionsRequestType,
        preferences: Preferences = .standard,
    ) {
        self.walletSessionService = walletSessionService
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
        walletSessionService.currentWallet
    }

    public var walletId: WalletId {
        wallet.id
    }

    public var currency: String {
        preferences.currency
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
