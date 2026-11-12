// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemAddressRow
import enum Gemstone.GemInfoTopic
import enum Gemstone.GemLoadState
import struct Gemstone.GemSwapAgain
import struct Gemstone.GemTransactionDetailRows
import struct Gemstone.GemTransactionDetailSection
import protocol Gemstone.GemTransactionDetailsServiceProtocol
import enum Gemstone.GemTransactionHeaderAction
import protocol Gemstone.GemTransactionsServiceProtocol
import func Gemstone.transactionDetailSections
import GemstonePrimitives
import InfoSheet
import Primitives
import PrimitivesComponents
import Store
import SwiftUI

@Observable
@MainActor
public final class TransactionSceneViewModel {
    private let onHeaderAction: ((GemTransactionHeaderAction) -> Void)?
    private let onAddContact: ((AddContactType) -> Void)?
    private let onSelectAddress: (@MainActor @Sendable (ChainAddress) -> Void)?
    private let walletId: WalletId
    private let transactionId: TransactionId
    private let transactionsService: any GemTransactionsServiceProtocol

    public let query: ObservableQuery<MappedQuery<TransactionQuery, TransactionDetails?>>
    private(set) var loadState: GemLoadState = .loading

    var isPresentingTransactionSheet: TransactionSheetType?

    public init(
        transactionId: TransactionId,
        transaction: TransactionExtended?,
        wallet: Wallet,
        service: any GemTransactionDetailsServiceProtocol,
        transactionsService: any GemTransactionsServiceProtocol,
        onHeaderAction: ((GemTransactionHeaderAction) -> Void)? = nil,
        onAddContact: ((AddContactType) -> Void)? = nil,
        onSelectAddress: (@MainActor @Sendable (ChainAddress) -> Void)? = nil,
    ) {
        self.onHeaderAction = onHeaderAction
        self.onAddContact = onAddContact
        self.onSelectAddress = onSelectAddress
        self.transactionId = transactionId
        self.transactionsService = transactionsService
        walletId = wallet.id
        let walletType = wallet.type.toGem()
        let details: @Sendable (TransactionExtended?) -> TransactionDetails? = { [service] transaction in
            transaction.map { TransactionDetails(transaction: $0, rows: service.detailRows(transaction: $0.toGem(), walletType: walletType)) }
        }
        let key: TransactionQuery.Key = transaction.map { .record($0.recordId) } ?? .transaction(transactionId)
        query = ObservableQuery(
            MappedQuery(TransactionQuery(walletId: wallet.id, key: key), transform: details),
            initialValue: details(transaction),
        )
    }

    var details: TransactionDetails? {
        query.value
    }

    var title: String {
        rows?.title.title ?? ""
    }

    var explorerLink: String? {
        rows?.explorer.link
    }

    var onTransactionHeaderTap: TransactionHeaderActionHandler? {
        guard onHeaderAction != nil, rows?.headerAction != nil else { return nil }
        return { [weak self] tap in self?.onHeaderTap(tap) }
    }

    func load() async {
        loadState = .loading
        loadState = await transactionsService.loadTransaction(walletId: walletId, transactionId: transactionId)
    }

    func onRecordChange(_ recordId: UInt64?) {
        guard let recordId else { return }
        query.request.base = TransactionQuery(walletId: walletId, key: .record(recordId))
    }
}

// MARK: - Sections

public extension TransactionSceneViewModel {
    var sections: [GemTransactionDetailSection] {
        rows.map { transactionDetailSections(rows: $0) } ?? []
    }
}

// MARK: - Actions

extension TransactionSceneViewModel {
    var addContactAction: ((AddContactType) -> Void)? {
        onAddContact
    }

    func selectAction(_ row: GemAddressRow) -> (@MainActor @Sendable () -> Void)? {
        guard let onSelectAddress else { return nil }
        let chainAddress = ChainAddress(chain: Chain(core: row.chain), address: row.address)
        return { onSelectAddress(chainAddress) }
    }

    private func onHeaderTap(_ tap: TransactionHeaderTap) {
        guard let onHeaderAction, let headerAction = rows?.headerAction else { return }
        switch tap {
        case .header:
            onHeaderAction(headerAction)
        case let .asset(assetId):
            onHeaderAction(.asset(assetId: assetId))
        }
    }

    func onSelectProviderContract(_ address: String) {
        guard let chain = details?.transaction.transaction.assetId.chain else { return }
        onSelectAddress?(ChainAddress(chain: chain, address: address))
    }

    func onSelectSwapAgain(_ swap: GemSwapAgain) {
        onHeaderAction?(.swap(fromAssetId: swap.fromAssetId, toAssetId: swap.toAssetId))
    }

    func onSelectShare() {
        isPresentingTransactionSheet = .share
    }

    func onSelectFeeDetails() {
        isPresentingTransactionSheet = .feeDetails
    }

    func onInfo(_ topic: GemInfoTopic) {
        isPresentingTransactionSheet = .info(topic.infoSheet)
    }
}

// MARK: - Private

extension TransactionSceneViewModel {
    private var rows: GemTransactionDetailRows? {
        details?.rows
    }

    var feeDetailsViewModel: NetworkFeeSceneViewModel? {
        rows.map { NetworkFeeSceneViewModel(screen: $0.feeRow.screen()) }
    }
}

public struct TransactionDetails: Equatable, Sendable {
    let transaction: TransactionExtended
    let rows: GemTransactionDetailRows
}
