// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import protocol Gemstone.GemTransactionDetailsServiceProtocol
import enum Gemstone.GemTransactionDetailRow
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
    private let service: any GemTransactionDetailsServiceProtocol
    private let onHeaderAction: ((TransactionHeaderAction) -> Void)?
    private let onAddContact: ((AddContactType) -> Void)?

    public let query: ObservableQuery<TransactionRequest>
    var transactionExtended: TransactionExtended {
        query.value
    }

    var isPresentingTransactionSheet: TransactionSheetType?
    private var rateDirection: AssetRateFormatter.Direction = .direct

    public init(
        transaction: TransactionExtended,
        walletId: WalletId,
        service: any GemTransactionDetailsServiceProtocol,
        onHeaderAction: ((TransactionHeaderAction) -> Void)? = nil,
        onAddContact: ((AddContactType) -> Void)? = nil,
    ) {
        self.service = service
        self.onHeaderAction = onHeaderAction
        self.onAddContact = onAddContact
        query = ObservableQuery(TransactionRequest(walletId: walletId, recordId: transaction.recordId), initialValue: transaction)
    }

    var title: String {
        rows.title.title
    }

    var explorerURL: URL {
        explorerViewModel.url
    }

    var onTransactionHeaderTap: TransactionHeaderActionHandler? {
        guard onHeaderAction != nil, headerAction != nil else { return nil }
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
        case .header: TransactionHeaderViewModel(header: rows.header, currency: service.getCurrency())
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
            )
        case .memo: TransactionMemoViewModel(transaction: transactionExtended.transaction)
        case .resource: TransactionResourceViewModel(resource: rows.resource)
        case .rate: TransactionRateViewModel(rate: rows.rate, direction: rateDirection)
        case .network: TransactionNetworkViewModel(chain: transactionExtended.asset.chain)
        case .pnl: TransactionPnlViewModel(pnl: rows.pnl)
        case .price: TransactionPriceViewModel(price: rows.price)
        case .provider: TransactionProviderViewModel(name: rows.providerName)
        case .fee: TransactionNetworkFeeViewModel(feeDisplay: rows.fee.display(currency: service.getCurrency(), formatter: .auto), onInfoAction: onSelectFee)
        case .explorer: explorerViewModel
        }
    }
}

// MARK: - Actions

extension TransactionSceneViewModel {
    private func handleHeaderTap(_ tap: TransactionHeaderTap) {
        guard let onHeaderAction, let headerAction else { return }
        switch tap {
        case .header:
            onHeaderAction(headerAction)
        case let .asset(assetId):
            onHeaderAction(.asset(assetId: assetId))
        }
    }

    func onSelectSwapAgain() {
        guard let onHeaderAction, case let .swap(fromAssetId, toAssetId) = headerAction else {
            return
        }
        onHeaderAction(.swap(fromAssetId: fromAssetId, toAssetId: toAssetId))
    }

    func switchRateDirection() {
        switch rateDirection {
        case .direct: rateDirection = .inverse
        case .inverse: rateDirection = .direct
        }
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
        let assetImage = TransactionViewModel(transaction: transactionExtended, currency: service.getCurrency()).assetImage
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
        service.detailRows(transaction: transactionExtended.map())
    }

    private var explorerViewModel: TransactionExplorerViewModel {
        TransactionExplorerViewModel(transactionLink: rows.explorer.map())
    }

    private var headerAction: TransactionHeaderAction? {
        switch rows.headerAction {
        case let .asset(assetId): .asset(assetId: Primitives.AssetId(core: assetId))
        case let .nft(assetId): .nft(assetId: Primitives.NFTAssetId(core: assetId))
        case let .swap(fromAssetId, toAssetId): .swap(fromAssetId: Primitives.AssetId(core: fromAssetId), toAssetId: Primitives.AssetId(core: toAssetId))
        case let .perpetual(assetId): .perpetual(assetId: Primitives.AssetId(core: assetId))
        case .none: nil
        }
    }

    var feeDetailsViewModel: NetworkFeeSceneViewModel {
        let fee = rows.fee
        return NetworkFeeSceneViewModel(
            feeAsset: fee.asset.map(),
            currency: Currency(core: service.getCurrency()),
            selection: .priority(priority: .normal),
            feeAssetPrice: fee.price.map { $0.map().mapToPrice() },
            feeAmount: BigInt(fee.value),
        )
    }
}
