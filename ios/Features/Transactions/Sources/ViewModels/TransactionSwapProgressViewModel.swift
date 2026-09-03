// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Formatters
import struct Gemstone.GemSwapProgress
import enum Gemstone.GemSwapProgressStep
import GemstonePrimitives
import Localization
import Primitives

struct TransactionSwapProgressViewModel {
    private let progress: GemSwapProgress?

    init(progress: GemSwapProgress?) {
        self.progress = progress
    }
}

// MARK: - ItemModelProvidable

extension TransactionSwapProgressViewModel: ItemModelProvidable {
    var itemModel: TransactionItemModel {
        guard let progress else {
            return .empty
        }
        let fromAsset = progress.fromAsset.map()
        let amount = ValueFormatter.auto.string(BigInt(core: progress.fromValue), asset: fromAsset)
        return .swapProgress(TransactionSwapProgressItemModel(
            transfer: .init(
                title: Localized.Transfer.title,
                subtitle: "\(amount) (\(fromAsset.id.chain.networkName))",
                status: progress.transfer.status,
            ),
            swap: .init(
                title: Localized.Wallet.swap,
                subtitle: progress.providerName,
                status: progress.swap.status,
            ),
            estimatedTime: progress.etaSeconds.map { EstimatedConfirmationFormatter().string(seconds: $0) },
        ))
    }
}

private extension GemSwapProgressStep {
    var status: TransactionSwapProgressItemModel.Step.Status {
        switch self {
        case .pending: .pending
        case .waiting: .waiting
        case .completed: .completed
        case .failed: .failed
        case .reverted: .reverted
        }
    }
}
