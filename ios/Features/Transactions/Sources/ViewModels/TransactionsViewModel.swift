// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemTransactionsServiceProtocol
import Components
import protocol Gemstone.GemExplorerServiceProtocol
import Foundation
import protocol Gemstone.GemPreferencesServiceProtocol
import Localization
import Primitives
import PrimitivesComponents
import Store
import GemstoneServices

@Observable
@MainActor
public final class TransactionsViewModel {
    public let explorerService: any GemExplorerServiceProtocol
    public let transactionsService: any GemTransactionsServiceProtocol
    private let preferencesService: any GemPreferencesServiceProtocol

    private let type: TransactionsRequestType

    public let wallet: Wallet

    public var transactions: [TransactionExtended] {
        filterModel.query.value
    }

    public var filterModel: TransactionsFilterViewModel

    public var isPresentingSheet: TransactionsSheetType?
    public var isPresentingToastMessage: ToastMessage?

    public init(
        transactionsService: any GemTransactionsServiceProtocol,
        explorerService: any GemExplorerServiceProtocol,
        wallet: Wallet,
        type: TransactionsRequestType,
        preferencesService: any GemPreferencesServiceProtocol,
    ) {
        self.transactionsService = transactionsService
        self.explorerService = explorerService
        self.type = type
        self.wallet = wallet
        filterModel = TransactionsFilterViewModel(wallet: wallet, type: type)
        self.preferencesService = preferencesService
    }

    public var title: String {
        Localized.Activity.title
    }

    public var walletId: WalletId {
        wallet.id
    }

    public var currency: String {
        preferencesService.currencyCode
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
            try await transactionsService.sync(walletId: walletId.id, assetId: nil)
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
