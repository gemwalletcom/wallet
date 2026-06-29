// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import ExplorerService
import Formatters
import Foundation
import InfoSheet
import Preferences
import Primitives
import PrimitivesComponents
import Store
import SwiftUI

@Observable
@MainActor
public final class TransactionSceneViewModel {
    private let preferences: Preferences
    private let explorerService: ExplorerService
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
        preferences: Preferences = Preferences.standard,
        explorerService: ExplorerService = ExplorerService.standard,
        onHeaderAction: ((TransactionHeaderAction) -> Void)? = nil,
        onAddContact: ((AddContactType) -> Void)? = nil,
    ) {
        self.preferences = preferences
        self.explorerService = explorerService
        self.onHeaderAction = onHeaderAction
        self.onAddContact = onAddContact
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
            ListSection(type: .details, [.date, .status, .participant, .memo, .rate, .network, .pnl, .price, .provider]),
            ListSection(type: .fee, [.fee]),
            ListSection(type: .explorer, [.explorerLink]),
        ]
    }

    public func itemModel(for item: TransactionItem) -> any ItemModelProvidable<TransactionItemModel> {
        switch item {
        case .header: headerViewModel
        case .swapProgress: TransactionSwapProgressViewModel(transaction: transactionExtended)
        case .swapButton: TransactionSwapButtonViewModel(metadata: model.transaction.transaction.metadata?.decode(TransactionSwapMetadata.self), state: model.transaction.transaction.state)
        case .date: TransactionDateViewModel(date: model.transaction.transaction.createdAt)
        case .status: TransactionStatusViewModel(state: model.transaction.transaction.state, onInfoAction: onSelectStatusInfo)
        case .participant: TransactionParticipantViewModel(transactionViewModel: model, onAddContact: onAddContact)
        case .memo: TransactionMemoViewModel(transaction: model.transaction.transaction)
        case .rate: TransactionRateViewModel(transaction: model.transaction, direction: rateDirection)
        case .network: TransactionNetworkViewModel(chain: model.transaction.asset.chain)
        case .pnl: TransactionPnlViewModel(metadata: model.transaction.transaction.metadata?.decode(TransactionPerpetualMetadata.self))
        case .price: TransactionPriceViewModel(metadata: model.transaction.transaction.metadata?.decode(TransactionPerpetualMetadata.self))
        case .provider: TransactionProviderViewModel(metadata: model.transaction.transaction.metadata?.decode(TransactionSwapMetadata.self))
        case .fee: TransactionNetworkFeeViewModel(feeDisplay: model.infoModel.feeDisplay, onInfoAction: onSelectFee)
        case .explorerLink: TransactionExplorerViewModel(transactionViewModel: model, explorerService: explorerService)
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
        let chain = model.transaction.transaction.assetId.chain
        isPresentingTransactionSheet = .info(.networkFee(chain))
    }

    private func onSelectStatusInfo() {
        let assetImage = model.assetImage
        isPresentingTransactionSheet = .info(.transactionState(
            imageURL: assetImage.imageURL,
            placeholder: assetImage.placeholder,
            state: model.transaction.transaction.state,
        ))
    }
}

// MARK: - Private

extension TransactionSceneViewModel {
    private var model: TransactionViewModel {
        TransactionViewModel(
            explorerService: explorerService,
            transaction: transactionExtended,
            currency: preferences.currency,
        )
    }

    private var headerViewModel: TransactionHeaderViewModel {
        TransactionHeaderViewModel(
            transaction: model.transaction,
            infoModel: model.infoModel,
        )
    }

    private var explorerViewModel: TransactionExplorerViewModel {
        TransactionExplorerViewModel(
            transactionViewModel: model,
            explorerService: explorerService,
        )
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
        let viewModel = NetworkFeeSceneViewModel(
            chain: model.transaction.transaction.assetId.chain,
            feeAsset: model.transaction.feeAsset,
            priority: .normal,
            currency: Currency(rawValue: preferences.currency) ?? .usd,
            feeAmount: BigInt(model.transaction.transaction.fee),
        )
        viewModel.update(rates: [], feeAssetPrice: model.transaction.feePrice)
        return viewModel
    }
}
