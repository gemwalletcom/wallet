// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Formatters
import Foundation
import enum Gemstone.GemInfoTopic
import enum Gemstone.GemTransactionDetailRow
import struct Gemstone.GemTransactionDetailRows
import protocol Gemstone.GemTransactionDetailsServiceProtocol
import enum Gemstone.GemTransactionHeaderAction
import func Gemstone.transactionDetailSections
import GemstonePrimitives
import InfoSheet
import Localization
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

    public let query: ObservableQuery<MappedRequest<TransactionRequest, TransactionDetails>>
    var transactionExtended: TransactionExtended {
        query.value.transaction
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
        let walletType = wallet.type.toGem()
        let details: @Sendable (TransactionExtended) -> TransactionDetails = { [service] in
            TransactionDetails(transaction: $0, rows: service.detailRows(transaction: $0.toGem(), walletType: walletType))
        }
        query = ObservableQuery(
            MappedRequest(TransactionRequest(walletId: wallet.id, recordId: transaction.recordId), transform: details),
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

// MARK: - ListSectionProvideable

extension TransactionSceneViewModel: ListSectionProvideable {
    public var sections: [ListSection<GemTransactionDetailRow>] {
        transactionDetailSections(rows: rows).map(ListSection.init)
    }

    public func itemModel(for row: GemTransactionDetailRow) -> any ItemModelProvidable<TransactionItemModel> {
        switch row {
        case .header: headerItem
        case .swapProgress: swapProgressItem
        case .swapAgain: rows.swapAgain == nil ? TransactionItemModel.empty : .swapAgain(text: Localized.Transaction.swapAgain)
        case .estimatedConfirmation: estimatedConfirmationItem
        case .participant: TransactionParticipantViewModel(
                participant: rows.participant,
                chain: transactionExtended.transaction.assetId.chain,
                memo: transactionExtended.transaction.memo,
                onAddContact: onAddContact,
                onSelectAddress: onSelectAddress,
            )
        case .rate: rows.rate.map { TransactionItemModel.rate(title: Localized.Buy.rate, value: AssetRateViewModel(rate: $0).text(isInverse: isRateInverse)) } ?? .empty
        case .fee: feeItem
        case let .row(row): TransactionItemModel.row(row)
        }
    }

    private var headerItem: TransactionItemModel {
        let headerType = rows.header.headerType(currency: service.getCurrency().toPrimitives())
        let showClearHeader = switch headerType {
        case .amount, .nft, .asset, .assetValue: true
        case .swap: false
        }
        return .header(TransactionHeaderItemModel(headerType: headerType, showClearHeader: showClearHeader))
    }

    private var swapProgressItem: TransactionItemModel {
        guard let progress = rows.swapProgress else { return .empty }
        let fromAsset = progress.fromAsset.toPrimitives()
        let amount = ValueFormatter.auto.string(BigInt(progress.fromValue), asset: fromAsset)
        return .swapProgress(TransactionSwapProgressItemModel(
            transfer: .init(title: Localized.Transfer.title, subtitle: progress.transferText(formattedValue: amount), state: progress.transfer),
            swap: .init(title: Localized.Wallet.swap, subtitle: progress.providerName, state: progress.swap),
            estimatedTime: progress.etaSeconds.map { EstimatedConfirmationFormatter().string(seconds: $0) },
        ))
    }

    private var estimatedConfirmationItem: TransactionItemModel {
        guard let seconds = rows.estimatedConfirmationSeconds else { return .empty }
        return .listItem(ListItemModel(
            title: Localized.Transaction.estimatedConfirmation,
            subtitle: EstimatedConfirmationFormatter().string(seconds: seconds),
            infoAction: onSelectEstimatedConfirmationInfo,
        ))
    }

    private var feeItem: TransactionItemModel {
        let fee = rows.feeRow
        return .fee(ListItemModel(
            title: fee.title.text,
            subtitle: fee.amount.text(),
            subtitleExtra: fee.fiat?.text(),
            infoAction: { [weak self] in self?.onInfo(fee.info) },
        ))
    }
}

// MARK: - Actions

extension TransactionSceneViewModel {
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

    func onInfo(_ topic: GemInfoTopic) {
        isPresentingTransactionSheet = .info(InfoSheetType(topic: topic, assetImage: TransactionViewModel(transaction: transactionExtended).assetImage))
    }

    private func onSelectEstimatedConfirmationInfo() {
        isPresentingTransactionSheet = .info(.estimatedConfirmation(transactionExtended.transaction.assetId.chain))
    }
}

// MARK: - Private

extension TransactionSceneViewModel {
    private var rows: GemTransactionDetailRows {
        query.value.rows
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

public struct TransactionDetails: Equatable, Sendable {
    let transaction: TransactionExtended
    let rows: GemTransactionDetailRows
}
