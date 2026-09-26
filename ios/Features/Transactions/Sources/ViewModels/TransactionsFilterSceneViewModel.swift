// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import func Gemstone.activityFilters
import enum Gemstone.GemTransactionFilter
import struct Gemstone.GemTransactionRow
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Store

@Observable
@MainActor
public final class TransactionsFilterSceneViewModel {
    private let wallet: Wallet
    private let type: TransactionsQueryType

    public var chainsFilter: ChainsFilterViewModel {
        didSet { query.request.base.filter = requestFilter }
    }

    public var transactionTypesFilter: TransactionTypesFilterViewModel {
        didSet { query.request.base.filter = requestFilter }
    }

    public let query: ObservableQuery<MappedQuery<TransactionsQuery, [ListSection<GemTransactionRow>]>>

    var isPresentingChains: Bool = false
    var isPresentingTypes: Bool = false

    public init(wallet: Wallet, chains: [Chain], type: TransactionsQueryType) {
        self.wallet = wallet
        self.type = type

        chainsFilter = ChainsFilterViewModel(chains: chains)
        transactionTypesFilter = TransactionTypesFilterViewModel()

        let request = TransactionsQuery(
            walletId: wallet.id,
            type: type,
            filter: activityFilters(chains: [], filters: []).toPrimitives(),
            limit: GemConstants.transactionsListLimit,
        )
        query = ObservableQuery(MappedQuery(request, transform: transactionListSections), initialValue: [])
    }

    public func onFinishChainsSelection(_ value: SelectionResult<Chain>) -> Bool {
        chainsFilter.selectedChains = value.items
        return value.isConfirmed
    }

    public func onFinishTypesSelection(_ value: SelectionResult<GemTransactionFilter>) -> Bool {
        transactionTypesFilter.selectedTypes = value.items
        return value.isConfirmed
    }

    public var isAnyFilterSpecified: Bool {
        chainsFilter.isAnySelected || transactionTypesFilter.isAnySelected
    }

    public var title: String {
        Localized.Filter.title
    }

    public var clear: String {
        Localized.Filter.clear
    }

    public var done: String {
        Localized.Common.done
    }

    public var networksModel: NetworkSelectorViewModel {
        NetworkSelectorViewModel(
            state: .data(.plain(chainsFilter.allChains)),
            selectedItems: chainsFilter.selectedChains,
            selectionType: .multiSelection,
        )
    }

    public var typesModel: TransactionTypesSelectorViewModel {
        TransactionTypesSelectorViewModel(
            state: .data(.plain(transactionTypesFilter.allTransactionsTypes)),
            selectedItems: transactionTypesFilter.selectedTypes,
            selectionType: .multiSelection,
        )
    }

    private var requestFilter: TransactionsFilter {
        activityFilters(
            chains: chainsFilter.selectedChains.map(\.rawValue),
            filters: transactionTypesFilter.selectedTypes,
        ).toPrimitives()
    }
}

// MARK: - Actions

extension TransactionsFilterSceneViewModel {
    func onSelectChainsFilter() {
        isPresentingChains = true
    }

    func onSelectTypesFilter() {
        isPresentingTypes = true
    }
}
