// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemLoadState
import protocol Gemstone.GemTransactionsServiceProtocol
import func Gemstone.loadError
import func Gemstone.transactionsEmptyState
import GemstonePrimitives
import GemstoneServices
import Localization
import Primitives
import PrimitivesComponents
import Store

@Observable
@MainActor
public final class TransactionsSceneViewModel {
    private let service: any GemTransactionsServiceProtocol
    private let type: TransactionsQueryType

    public let wallet: Wallet

    public var sections: [ListSection<TransactionViewModel>] {
        filterModel.query.value
    }

    public var filterModel: TransactionsFilterSceneViewModel

    public var isPresentingSheet: TransactionsSheetType?
    public var isPresentingToastMessage: ToastMessage?

    private var transactionsState: GemLoadState = .loading

    public init(
        service: any GemTransactionsServiceProtocol,
        wallet: Wallet,
        type: TransactionsQueryType,
    ) {
        self.service = service
        self.type = type
        self.wallet = wallet
        filterModel = TransactionsFilterSceneViewModel(wallet: wallet, chains: Self.filterChains(service, wallet: wallet), type: type)
    }

    private static func filterChains(_ service: any GemTransactionsServiceProtocol, wallet: Wallet) -> [Chain] {
        service.filterChains(wallet: wallet.toGem()).map { Chain(core: $0) }
    }

    public var title: String {
        Localized.Activity.title
    }

    public var walletId: WalletId {
        wallet.id
    }

    public var loadError: Error? {
        Gemstone.loadError(state: transactionsState, hasRows: !sections.isEmpty)
    }

    public var emptyContentModel: EmptyContentTypeViewModel {
        let kind = transactionsEmptyState(
            chains: filterModel.chainsFilter.selectedChains.map(\.rawValue),
            filters: filterModel.transactionTypesFilter.selectedTypes,
        )
        return EmptyContentTypeViewModel(type: EmptyContentType(kind, isViewOnly: wallet.isViewOnly, actions: [.buy: onSelectBuy, .receive: onSelectReceive, .clearFilters: onSelectCleanFilters]))
    }
}

// MARK: - Business Logic

public extension TransactionsSceneViewModel {
    func onSelectFilterButton() {
        isPresentingSheet = .filter
    }

    func load() async {
        transactionsState = await service.refresh(assetId: nil, hasTransactions: sections.isNotEmpty)
    }
}

// MARK: - Private

extension TransactionsSceneViewModel {
    private func onSelectCleanFilters() {
        filterModel = TransactionsFilterSceneViewModel(wallet: wallet, chains: Self.filterChains(service, wallet: wallet), type: type)
    }

    private func onSelectReceive() {
        isPresentingSheet = .selectAsset(.receive(.asset))
    }

    private func onSelectBuy() {
        isPresentingSheet = .selectAsset(.buy)
    }
}
