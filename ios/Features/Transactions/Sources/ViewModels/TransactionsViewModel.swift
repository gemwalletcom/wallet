// Copyright (c). Gem Wallet. All rights reserved.

import Components
import protocol Gemstone.GemExplorerServiceProtocol
import Foundation
import Localization
import Preferences
import Primitives
import PrimitivesComponents
import Store
import GemstoneServices

@Observable
@MainActor
public final class TransactionsViewModel {
    public let explorerService: any GemExplorerServiceProtocol
    public let transactionsService: TransactionsService
    public let preferences: Preferences

    private let type: TransactionsRequestType

    public let wallet: Wallet

    public var transactions: [TransactionExtended] {
        filterModel.query.value
    }

    public var filterModel: TransactionsFilterViewModel

    public var isPresentingSheet: TransactionsSheetType?
    public var isPresentingToastMessage: ToastMessage?

    public init(
        transactionsService: TransactionsService,
        explorerService: any GemExplorerServiceProtocol,
        wallet: Wallet,
        type: TransactionsRequestType,
        preferences: Preferences = .standard,
    ) {
        self.transactionsService = transactionsService
        self.explorerService = explorerService
        self.type = type
        self.wallet = wallet
        filterModel = TransactionsFilterViewModel(wallet: wallet, type: type)
        self.preferences = preferences
    }

    public var title: String {
        Localized.Activity.title
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
    private func onSelectCleanFilters() {
        filterModel = TransactionsFilterViewModel(wallet: wallet, type: type)
    }

    private func onSelectReceive() {
        isPresentingSheet = .selectAsset(.receive(.asset))
    }

    private func onSelectBuy() {
        isPresentingSheet = .selectAsset(.buy)
    }
}
