// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import protocol Gemstone.GemTransactionDetailsServiceProtocol
import enum Gemstone.GemTransactionDetailRow
import enum Gemstone.GemTransactionHeaderAction
import struct Gemstone.GemTransactionDetailRows
import func Gemstone.transactionDetailSections
import GemstonePrimitives
import Formatters
import Foundation
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

    public let query: ObservableQuery<TransactionRequest>
    var transactionExtended: TransactionExtended {
        query.value
    }

    var isPresentingTransactionSheet: TransactionSheetType?
    private var isRateInverse = false

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
        query = ObservableQuery(TransactionRequest(walletId: wallet.id, recordId: transaction.recordId), initialValue: transaction)
    }

    var title: String {
        rows.title.title
    }

    var explorerURL: URL {
        explorerViewModel.url
    }

    var onTransactionHeaderTap: TransactionHeaderActionHandler? {
        guard onHeaderAction != nil, rows.headerAction != nil else { return nil }
        return { [weak self] tap in self?.handleHeaderTap(tap) }
    }
}

// MARK: - ListSectionProvideable

extension TransactionSceneViewModel: ListSectionProvideable {
    public var sections: [ListSection<GemTransactionDetailRow>] {
        transactionDetailSections(rows: rows).map(ListSection.init)
    }

    public func itemModel(for row: GemTransactionDetailRow) -> any ItemModelProvidable<TransactionItemModel> {
        switch row {
        case .header: TransactionHeaderViewModel(header: rows.header, currency: service.getCurrency().toPrimitives())
        case .swapProgress: TransactionSwapProgressViewModel(progress: rows.swapProgress)
        case .swapAgain: TransactionSwapButtonViewModel(swapAgain: rows.swapAgain)
        case .date: TransactionDateViewModel(date: transactionExtended.transaction.createdAt)
        case .status: TransactionStatusViewModel(status: rows.status, state: transactionExtended.transaction.state, onInfoAction: onSelectStatusInfo)
        case .estimatedConfirmation: TransactionEstimatedConfirmationViewModel(seconds: rows.estimatedConfirmationSeconds, onInfoAction: onSelectEstimatedConfirmationInfo)
        case .participant: TransactionParticipantViewModel(
                participant: rows.participant,
                chain: transactionExtended.transaction.assetId.chain,
                memo: transactionExtended.transaction.memo,
                onAddContact: onAddContact,
                onSelectAddress: onSelectAddress,
            )
        case .memo: TransactionMemoViewModel(transaction: transactionExtended.transaction)
        case .resource: TransactionResourceViewModel(resource: rows.resource)
        case .rate: TransactionRateViewModel(rate: rows.rate, isInverse: isRateInverse)
        case .network: TransactionNetworkViewModel(chain: transactionExtended.asset.chain)
        case .pnl: TransactionPnlViewModel(pnl: rows.pnl)
        case .price: TransactionPriceViewModel(price: rows.price)
        case .provider: TransactionProviderViewModel(name: rows.providerName)
        case .fee: TransactionNetworkFeeViewModel(feeDisplay: rows.fee.display(currency: service.getCurrency().toPrimitives(), formatter: .auto), onInfoAction: onSelectFee)
        case .explorer: explorerViewModel
        }
    }
}

// MARK: - Actions

extension TransactionSceneViewModel {
    private func handleHeaderTap(_ tap: TransactionHeaderTap) {
        guard let onHeaderAction, let headerAction = rows.headerAction else { return }
        switch tap {
        case .header:
            onHeaderAction(headerAction)
        case let .asset(assetId):
            onHeaderAction(.asset(assetId: assetId.identifier))
        }
    }

    func onSelectSwapAgain() {
        guard let onHeaderAction, case let .swap(fromAssetId, toAssetId) = rows.headerAction else {
            return
        }
        onHeaderAction(.swap(fromAssetId: fromAssetId, toAssetId: toAssetId))
    }

    func switchRateDirection() {
        isRateInverse.toggle()
    }

    func onSelectShare() {
        isPresentingTransactionSheet = .share
    }

    func onSelectFeeDetails() {
        isPresentingTransactionSheet = .feeDetails
    }

    private func onSelectFee() {
        isPresentingTransactionSheet = .info(.networkFee(transactionExtended.feeAsset))
    }

    private func onSelectStatusInfo() {
        let assetImage = TransactionViewModel(transaction: transactionExtended).assetImage
        isPresentingTransactionSheet = .info(.transactionState(
            imageURL: assetImage.imageURL,
            placeholder: assetImage.placeholder,
            model: TransactionStateViewModel(state: transactionExtended.transaction.state, tone: rows.status.tone),
        ))
    }

    private func onSelectEstimatedConfirmationInfo() {
        isPresentingTransactionSheet = .info(.estimatedConfirmation(transactionExtended.transaction.assetId.chain))
    }
}

// MARK: - Private

extension TransactionSceneViewModel {
    private var rows: GemTransactionDetailRows {
        service.detailRows(transaction: transactionExtended.toGem(), walletType: wallet.type.toGem())
    }

    private var explorerViewModel: TransactionExplorerViewModel {
        TransactionExplorerViewModel(transactionLink: rows.explorer.toPrimitives())
    }

    var feeDetailsViewModel: NetworkFeeSceneViewModel {
        let fee = rows.fee
        return NetworkFeeSceneViewModel(
            feeAsset: fee.asset.toPrimitives(),
            currency: service.getCurrency().toPrimitives(),
            selection: .priority(priority: .normal),
            feeAssetPrice: fee.price.map { $0.toPrimitives().mapToPrice() },
            feeAmount: BigInt(fee.value),
        )
    }
}
