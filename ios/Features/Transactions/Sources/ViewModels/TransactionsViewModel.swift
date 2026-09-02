// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemChainServiceProtocol
import protocol Gemstone.GemTransactionsServiceProtocol
import Components
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
    public let transactionsService: any GemTransactionsServiceProtocol
    private let preferencesService: any GemPreferencesServiceProtocol

    private let type: TransactionsRequestType
    private let chainService: any GemChainServiceProtocol

    public let wallet: Wallet

    public var transactions: [TransactionExtended] {
        filterModel.query.value
    }

    public var filterModel: TransactionsFilterViewModel

    public var isPresentingSheet: TransactionsSheetType?
    public var isPresentingToastMessage: ToastMessage?

    public init(
        transactionsService: any GemTransactionsServiceProtocol,
        wallet: Wallet,
        type: TransactionsRequestType,
        chainService: any GemChainServiceProtocol,
        preferencesService: any GemPreferencesServiceProtocol,
    ) {
        self.transactionsService = transactionsService
        self.type = type
        self.chainService = chainService
        self.wallet = wallet
        filterModel = TransactionsFilterViewModel(wallet: wallet, type: type, chainService: chainService)
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

    func load() async {
        do {
            try await transactionsService.sync(walletId: walletId.id, assetId: nil)
        } catch {
            debugLog("load getTransactions error \(error)")
        }
    }
}

// MARK: - Private

extension TransactionsViewModel {
    private func onSelectCleanFilters() {
        filterModel = TransactionsFilterViewModel(wallet: wallet, type: type, chainService: chainService)
    }

    private func onSelectReceive() {
        isPresentingSheet = .selectAsset(.receive(.asset))
    }

    private func onSelectBuy() {
        isPresentingSheet = .selectAsset(.buy)
    }
}
