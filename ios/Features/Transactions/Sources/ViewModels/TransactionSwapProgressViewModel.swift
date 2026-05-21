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
        guard let progress = progressModel else {
            return .empty
        }
        return .swapProgress(progress)
    }
}

// MARK: - Private

extension TransactionSwapProgressViewModel {
    private var progressModel: TransactionSwapProgressItemModel? {
        guard
            transaction.transaction.type == .swap,
            let metadata = transaction.transaction.metadata?.decode(TransactionSwapMetadata.self),
            let providerId = metadata.provider,
            let swapProvider = SwapProvider(rawValue: providerId),
            let fromAsset = transaction.assets.first(where: { $0.id == metadata.fromAsset })
        else {
            return nil
        }

        let provider = SwapProviderConfig.fromString(id: swapProvider.rawValue).inner()
        guard provider.mode.isCrossChain else {
            return nil
        }

        let transferTitle = Localized.Transfer.title
        let chainName = Asset(fromAsset.id.chain).name
        let amount = ValueFormatter.auto.string(BigInt.fromString(metadata.fromValue), asset: fromAsset)
        let transferSubtitle = "\(amount) (\(chainName))"
        let swapTitle = Localized.Wallet.swap
        let swapSubtitle = provider.name

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
            transfer: .init(title: transferTitle, subtitle: transferSubtitle, status: transferStatus),
            swap: .init(title: swapTitle, subtitle: swapSubtitle, status: swapStatus),
        )
    }
}

private extension SwapperProviderMode {
    var isCrossChain: Bool {
        switch self {
        case .crossChain, .bridge, .omniChain: true
        case .onChain: false
        }
    }
}
