// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import PrimitivesComponents

struct ConfirmHeaderViewModel {
    private let request: ConfirmTransferRequest
    private let state: ConfirmTransferState

    init(
        request: ConfirmTransferRequest,
        state: ConfirmTransferState,
    ) {
        self.request = request
        self.state = state
    }
}

// MARK: - ItemModelProvidable

extension ConfirmHeaderViewModel: ItemModelProvidable {
    var itemModel: ConfirmTransferItemModel {
        .header(
            TransactionHeaderItemModel(
                headerType: headerType,
                showClearHeader: headerType.showsClearHeader,
            ),
        )
    }
}

extension TransactionHeaderType {
    var showsClearHeader: Bool {
        switch self {
        case .amount, .nft, .asset, .assetValue: true
        case .swap: false
        }
    }
}

// MARK: - Private

private extension ConfirmHeaderViewModel {
    var headerType: TransactionHeaderType {
        if let headerData = state.simulation.headerData {
            return .assetValue(headerData)
        }

        if case let .tokenApprove(asset, _) = request.data.type {
            return .asset(image: AssetViewModel(asset: asset).assetImage)
        }

        if case .generic = request.data.type,
           let header = request.simulation?.header
        {
            return .asset(image: AssetIdViewModel(assetId: header.assetId).assetImage)
        }

        return TransactionInputViewModel(
            data: request.data,
            transactionData: state.transaction.value?.transactionData,
            metaData: state.metadata,
            transferAmount: state.transaction.value?.transferAmount,
        ).headerType
    }
}
