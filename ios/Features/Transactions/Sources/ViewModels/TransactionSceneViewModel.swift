// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import struct Gemstone.GemAddressRow
import enum Gemstone.GemInfoTopic
import struct Gemstone.GemSwapAgain
import enum Gemstone.GemTransactionDetailRow
import struct Gemstone.GemTransactionDetailRows
import protocol Gemstone.GemTransactionDetailsServiceProtocol
import enum Gemstone.GemTransactionHeaderAction
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
    private let wallet: Wallet
    private let service: any GemTransactionDetailsServiceProtocol
    private let onHeaderAction: ((GemTransactionHeaderAction) -> Void)?
    private let onAddContact: ((AddContactType) -> Void)?
    private let onSelectAddress: (@MainActor @Sendable (ChainAddress) -> Void)?

    public let query: ObservableQuery<MappedQuery<TransactionQuery, TransactionDetails>>
    var transactionExtended: TransactionExtended {
        query.value.transaction
    }

    var isPresentingTransactionSheet: TransactionSheetType?

    public init(
        transaction: TransactionExtended,
        wallet: Wallet,
        service: any GemTransactionDetailsServiceProtocol,
        onHeaderAction: ((GemTransactionHeaderAction) -> Void)? = nil,
        onAddContact: ((AddContactType) -> Void)? = nil,
        onSelectAddress: (@MainActor @Sendable (ChainAddress) -> Void)? = nil,
    ) {
        self.wallet = wallet
        self.service = service
        self.onHeaderAction = onHeaderAction
        self.onAddContact = onAddContact
        self.onSelectAddress = onSelectAddress
        let walletType = wallet.type.toGem()
        let details: @Sendable (TransactionExtended) -> TransactionDetails = { [service] in
            TransactionDetails(transaction: $0, rows: service.detailRows(transaction: $0.toGem(), walletType: walletType))
        }
        query = ObservableQuery(
            MappedQuery(TransactionQuery(walletId: wallet.id, recordId: transaction.recordId), transform: details),
            initialValue: details(transaction),
        )
    }

    var title: String {
        rows.title.title
    }

    var explorerURL: URL {
        rows.explorer.toPrimitives().url
    }

    var onTransactionHeaderTap: TransactionHeaderActionHandler? {
        guard onHeaderAction != nil, rows.headerAction != nil else { return nil }
        return { [weak self] tap in self?.onHeaderTap(tap) }
    }
}

// MARK: - Sections

public extension TransactionSceneViewModel {
    var sections: [ListSection<GemTransactionDetailRow>] {
        transactionDetailSections(rows: rows).map(ListSection.init)
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
        guard let onHeaderAction, let headerAction = rows.headerAction else { return }
        switch tap {
        case .header:
            onHeaderAction(headerAction)
        case let .asset(assetId):
            onHeaderAction(.asset(assetId: assetId.identifier))
        }
    }

    func onSelectProviderContract(_ address: String) {
        onSelectAddress?(ChainAddress(chain: transactionExtended.transaction.assetId.chain, address: address))
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
    private var rows: GemTransactionDetailRows {
        query.value.rows
    }

    var feeDetailsViewModel: NetworkFeeSceneViewModel {
        NetworkFeeSceneViewModel(screen: rows.feeRow.screen())
    }
}

public struct TransactionDetails: Equatable, Sendable {
    let transaction: TransactionExtended
    let rows: GemTransactionDetailRows
}
