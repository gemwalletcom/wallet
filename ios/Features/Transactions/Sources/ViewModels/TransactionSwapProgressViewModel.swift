// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Formatters
import Gemstone
import GemstonePrimitives
import Localization
import Primitives

struct TransactionSwapProgressViewModel {
    private let transaction: TransactionExtended

    init(transaction: TransactionExtended) {
        self.transaction = transaction
    }
}

// MARK: - ItemModelProvidable

extension TransactionSwapProgressViewModel: ItemModelProvidable {
    var itemModel: TransactionItemModel {
        guard let progress else {
            return .empty
        }
        return .swapProgress(progress)
    }
}

// MARK: - Private

extension TransactionSwapProgressViewModel {
    var progress: TransactionSwapProgressItemModel? {
        guard
            transaction.transaction.type == .swap,
            let metadata = transaction.transaction.metadata?.decode(TransactionSwapMetadata.self),
            let providerId = metadata.provider,
            let swapProvider = swapperProviderFromStr(s: providerId),
            let fromAsset = transaction.assets.first(where: { $0.id == metadata.fromAsset }),
            let fromValue = try? BigInt.from(string: metadata.fromValue)
        else {
            return nil
        }

        let provider = swapperProviderConfig(provider: swapProvider)
        guard provider.mode != .onChain else {
            return nil
        }

        let transferTitle = Localized.Transfer.title
        let chainName = fromAsset.id.chain.networkName
        let amount = ValueFormatter.auto.string(fromValue, asset: fromAsset)
        let transferSubtitle = "\(amount) (\(chainName))"
        let swapTitle = Localized.Wallet.swap
        let swapSubtitle = provider.name
        let estimatedTime = transaction.transaction.state.isCompleted
            ? nil
            : transaction.confirmationEtaSeconds.flatMap { $0 > 0 ? EstimatedConfirmationFormatter().string(seconds: $0) : nil }

        let transferStatus: TransactionSwapProgressItemModel.Step.Status
        let swapStatus: TransactionSwapProgressItemModel.Step.Status
        switch transaction.transaction.state {
        case .pending:
            transferStatus = .pending
            swapStatus = .waiting
        case .inTransit:
            transferStatus = .completed
            swapStatus = .pending
        case .confirmed:
            return nil
        case .failed:
            transferStatus = .completed
            swapStatus = .failed
        case .reverted:
            transferStatus = .reverted
            swapStatus = .waiting
        }

        return TransactionSwapProgressItemModel(
            transfer: .init(
                title: transferTitle,
                subtitle: transferSubtitle,
                status: transferStatus,
            ),
            swap: .init(
                title: swapTitle,
                subtitle: swapSubtitle,
                status: swapStatus,
            ),
            estimatedTime: estimatedTime,
        )
    }
}
