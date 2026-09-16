// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import func Gemstone.transactionsListLimit
import Localization
import Primitives
import PrimitivesComponents
import Store
import enum Gemstone.GemTransactionFilter

@Observable
@MainActor
public final class TransactionsFilterViewModel {
    private let wallet: Wallet
    private let type: TransactionsRequestType

    public var chainsFilter: ChainsFilterViewModel {
        didSet { query.request.base.filters = requestFilters }
    }

    public var transactionTypesFilter: TransactionTypesFilterViewModel {
        didSet { query.request.base.filters = requestFilters }
    }

    public let query: ObservableQuery<MappedRequest<TransactionsRequest, [ListSection<TransactionViewModel>]>>

    var isPresentingChains: Bool = false
    var isPresentingTypes: Bool = false

    public init(wallet: Wallet, chains: [Chain], type: TransactionsRequestType) {
        self.wallet = wallet
        self.type = type

        chainsFilter = ChainsFilterViewModel(chains: chains)
        transactionTypesFilter = TransactionTypesFilterViewModel()

        let request = TransactionsRequest(
            walletId: wallet.id,
            type: type,
            filters: TransactionsRequestFilter.activity(chains: [], filters: []),
            limit: Int(transactionsListLimit()),
        )
        query = ObservableQuery(MappedRequest(request, transform: TransactionViewModel.sections), initialValue: [])
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

    private var requestFilters: [TransactionsRequestFilter] {
        TransactionsRequestFilter.activity(
            chains: chainsFilter.selectedChains,
            filters: transactionTypesFilter.selectedTypes,
        )
    }
}

// MARK: - Actions

extension TransactionsFilterViewModel {
    func onSelectChainsFilter() {
        isPresentingChains = true
    }

    func onSelectTypesFilter() {
        isPresentingTypes = true
    }
}
