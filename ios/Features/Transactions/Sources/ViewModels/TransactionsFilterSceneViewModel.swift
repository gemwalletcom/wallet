// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemTransactionFilter
import struct Gemstone.GemTransactionRow
import struct Gemstone.GemTransactionsFilterSession
import struct Gemstone.GemTransactionsFilterView
import func Gemstone.newTransactionsFilterSession
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Store

@Observable
@MainActor
public final class TransactionsFilterSceneViewModel {
    private var session: GemTransactionsFilterSession
    public private(set) var viewState: GemTransactionsFilterView

    public let query: ObservableQuery<MappedQuery<TransactionsQuery, [ListSection<GemTransactionRow>]>>

    var isPresentingChains: Bool = false
    var isPresentingTypes: Bool = false

    public init(wallet: Wallet, chains: [Chain], type: TransactionsQueryType) {
        let session = newTransactionsFilterSession(chains: chains.map(\.rawValue), walletType: wallet.type.toGem())
        let viewState = session.viewState()
        self.session = session
        self.viewState = viewState

        let request = TransactionsQuery(
            walletId: wallet.id,
            type: type,
            filter: viewState.filter.toPrimitives(),
            limit: GemConstants.transactionsListLimit,
        )
        query = ObservableQuery(MappedQuery(request, transform: transactionListSections), initialValue: [])
    }

    public func onFinishChainsSelection(_ value: SelectionResult<Chain>) -> Bool {
        update(session.onChains(chains: value.items.map(\.rawValue)))
        return value.isConfirmed
    }

    public func onFinishTypesSelection(_ value: SelectionResult<GemTransactionFilter>) -> Bool {
        update(session.onTypes(types: value.items))
        return value.isConfirmed
    }

    public func onClear() {
        update(session.onClear())
    }

    public var isAnyFilterSpecified: Bool {
        viewState.isFiltered
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

    public var chainsTypeModel: ChainsFilterTypeViewModel {
        ChainsFilterTypeViewModel(summary: viewState.chainsSummary)
    }

    public var typesTypeModel: TransactionsFilterTypeViewModel {
        TransactionsFilterTypeViewModel(summary: viewState.typesSummary)
    }

    public var networksModel: NetworkSelectorViewModel {
        NetworkSelectorViewModel(
            state: .data(.plain(session.chains.map { Chain(core: $0) })),
            selectedItems: session.selectedChains.map { Chain(core: $0) },
            selectionType: .multiSelection,
        )
    }

    public var typesModel: TransactionTypesSelectorViewModel {
        TransactionTypesSelectorViewModel(
            state: .data(.plain(viewState.types)),
            selectedItems: session.selectedTypes,
            selectionType: .multiSelection,
        )
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

// MARK: - Private

extension TransactionsFilterSceneViewModel {
    private func update(_ session: GemTransactionsFilterSession) {
        self.session = session
        viewState = session.viewState()
        query.request.base.filter = viewState.filter.toPrimitives()
    }
}
