// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemLoadState
import struct Gemstone.GemTransactionRow
import protocol Gemstone.GemTransactionsServiceProtocol
import func Gemstone.loadError
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

    public let wallet: Wallet

    public var sections: [ListSection<GemTransactionRow>] {
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

    public var emptyContentModel: EmptyStateViewModel {
        EmptyStateViewModel(state: filterModel.viewState.emptyState) { [weak self] action in
            switch action {
            case .buy: self?.onSelectBuy()
            case .receive: self?.onSelectReceive()
            case .clearFilters: self?.onSelectCleanFilters()
            case .swap, .addCustomToken, .manageTokenList: break
            }
        }
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
        filterModel.onClear()
    }

    private func onSelectReceive() {
        isPresentingSheet = .selectAsset(.receive(.asset))
    }

    private func onSelectBuy() {
        isPresentingSheet = .selectAsset(.buy)
    }
}
