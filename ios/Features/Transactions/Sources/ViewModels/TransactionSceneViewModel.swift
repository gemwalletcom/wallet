// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import protocol Gemstone.GemTransactionDetailsServiceProtocol
import struct Gemstone.GemTransactionDetails
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
    private let initialTransaction: TransactionExtended
    var transactionExtended: TransactionExtended {
        query.value ?? initialTransaction
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
        initialTransaction = transaction
        query = ObservableQuery(TransactionRequest(walletId: walletId, transactionId: transaction.transaction.id), initialValue: transaction)
    }

    var title: String {
        model.titleTextValue.text
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
    public var sections: [ListSection<TransactionItem>] {
        [
            ListSection(type: .header, [.header]),
            ListSection(type: .swapProgress, [.swapProgress]),
            ListSection(type: .swapAction, [.swapButton]),
            ListSection(type: .details, [.date, .status, .estimatedConfirmation, .participant, .memo, .rate, .network, .pnl, .price, .provider]),
            ListSection(type: .fee, [.fee]),
            ListSection(type: .explorer, [.explorerLink]),
        ]
    }

    public func itemModel(for item: TransactionItem) -> any ItemModelProvidable<TransactionItemModel> {
        switch item {
        case .header: headerViewModel
        case .swapProgress: TransactionSwapProgressViewModel(progress: details.swapProgress)
        case .swapButton: TransactionSwapButtonViewModel(swapAgain: details.swapAgain)
        case .date: TransactionDateViewModel(date: model.transaction.transaction.createdAt)
        case .status: TransactionStatusViewModel(state: model.transaction.transaction.state, onInfoAction: onSelectStatusInfo)
        case .estimatedConfirmation: TransactionEstimatedConfirmationViewModel(seconds: details.estimatedConfirmationSeconds, onInfoAction: onSelectEstimatedConfirmationInfo)
        case .participant: TransactionParticipantViewModel(transactionViewModel: model, participant: service.participant(transaction: transactionExtended.transaction.json()), onAddContact: onAddContact)
        case .memo: TransactionMemoViewModel(transaction: model.transaction.transaction)
        case .rate: TransactionRateViewModel(transaction: model.transaction, direction: rateDirection)
        case .network: TransactionNetworkViewModel(chain: model.transaction.asset.chain)
        case .pnl: TransactionPnlViewModel(pnl: details.pnl)
        case .price: TransactionPriceViewModel(price: details.price)
        case .provider: TransactionProviderViewModel(name: details.providerName)
        case .fee: TransactionNetworkFeeViewModel(feeDisplay: model.infoModel.feeDisplay, onInfoAction: onSelectFee)
        case .explorerLink: explorerViewModel
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
        isPresentingTransactionSheet = .info(.networkFee(model.transaction.feeAsset))
    }

    private func onSelectStatusInfo() {
        let assetImage = model.assetImage
        isPresentingTransactionSheet = .info(.transactionState(
            imageURL: assetImage.imageURL,
            placeholder: assetImage.placeholder,
            state: model.transaction.transaction.state,
        ))
    }

    private func onSelectEstimatedConfirmationInfo() {
        isPresentingTransactionSheet = .info(.estimatedConfirmation(model.transaction.transaction.assetId.chain))
    }
}

// MARK: - Private

extension TransactionSceneViewModel {
    private var model: TransactionViewModel {
        TransactionViewModel(transaction: transactionExtended, currency: service.getCurrency())
    }

    private var transactionLink: BlockExplorerLink {
        let transaction = transactionExtended.transaction
        return BlockExplorerLink(service.transactionLink(
            chain: transaction.assetId.chain.rawValue,
            hash: transaction.id.hash,
            provider: transaction.swapProvider,
            recipient: transaction.to,
            memo: transaction.memo,
        ))
    }

    private var headerViewModel: TransactionHeaderViewModel {
        TransactionHeaderViewModel(
            transaction: model.transaction,
            kind: service.headerKind(transaction: transactionExtended.transaction.json()),
            infoModel: model.infoModel,
        )
    }

    private var details: GemTransactionDetails {
        service.details(transaction: transactionExtended.json())
    }

    private var explorerViewModel: TransactionExplorerViewModel {
        TransactionExplorerViewModel(transactionLink: transactionLink)
    }

    private var headerAction: TransactionHeaderAction? {
        switch transactionExtended.transaction.type {
        case .transfer,
             .tokenApproval,
             .stakeDelegate,
             .stakeUndelegate,
             .stakeRewards,
             .stakeRedelegate,
             .stakeWithdraw,
             .stakeFreeze,
             .stakeUnfreeze:
            .asset(assetId: transactionExtended.transaction.assetId)
        case .transferNFT:
            transactionExtended.transaction.metadata?
                .decode(TransactionNFTTransferMetadata.self)
                .map { .nft(assetId: $0.assetId) }
        case .swap:
            transactionExtended.transaction.metadata?
                .decode(TransactionSwapMetadata.self)
                .map { .swap(fromAssetId: $0.fromAsset, toAssetId: $0.toAsset) }
        case .perpetualOpenPosition,
             .perpetualClosePosition,
             .perpetualModifyPosition:
            .perpetual(assetId: transactionExtended.transaction.assetId)
        case .smartContractCall,
             .assetActivation,
             .earnDeposit,
             .earnWithdraw:
            nil
        }
    }

    var feeDetailsViewModel: NetworkFeeSceneViewModel {
        NetworkFeeSceneViewModel(
            feeAsset: model.transaction.feeAsset,
            currency: Currency(core: service.getCurrency()),
            selection: .priority(priority: .normal),
            feeAssetPrice: model.transaction.feePrice,
            feeAmount: BigInt(model.transaction.transaction.fee),
        )
    }
}
