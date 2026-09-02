// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemTransactionsServiceProtocol
import Components
import Foundation
import Localization
import Primitives
import PrimitivesComponents
import Store
import GemstoneServices

@Observable
@MainActor
public final class TransactionsViewModel {
    private let service: any GemTransactionsServiceProtocol
    private let type: TransactionsRequestType

    public let wallet: Wallet

    public var transactions: [TransactionExtended] {
        filterModel.query.value
    }

    public var filterModel: TransactionsFilterViewModel

    public var isPresentingSheet: TransactionsSheetType?
    public var isPresentingToastMessage: ToastMessage?

    public init(
        service: any GemTransactionsServiceProtocol,
        wallet: Wallet,
        type: TransactionsRequestType,
    ) {
        self.service = service
        self.type = type
        self.wallet = wallet
        filterModel = TransactionsFilterViewModel(wallet: wallet, type: type)
    }

    public var title: String {
        Localized.Activity.title
    }

    public var walletId: WalletId {
        wallet.id
    }

    public var currency: String {
        service.currency()
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
            try await service.sync(assetId: nil)
        } catch {
            debugLog("load getTransactions error \(error)")
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
