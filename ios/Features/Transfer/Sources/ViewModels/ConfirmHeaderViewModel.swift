// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import PrimitivesComponents

struct ConfirmHeaderViewModel {
    private let state: ConfirmTransferState
    private let currency: Currency

    init(state: ConfirmTransferState, currency: Currency) {
        self.state = state
        self.currency = currency
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
        case .amount, .payment, .nft, .asset, .assetValue: true
        case .swap: false
        }
    }
}

// MARK: - Private

private extension ConfirmHeaderViewModel {
    var headerType: TransactionHeaderType {
        if let headerData = state.simulation.headerData {
            return .assetValue(AssetValueHeaderViewModel(data: headerData))
        }

        if case let .tokenApprove(asset, _) = state.transfer.inputType {
            return .asset(image: AssetViewModel(asset: asset.toPrimitives()).assetImage)
        }

        if case .generic = state.transfer.inputType,
           let header = state.simulation.result?.header
        {
            return .asset(image: AssetIdViewModel(assetId: AssetId(core: header.assetId)).assetImage)
        }

        return TransactionInputViewModel(
            data: state.transfer,
            fee: state.fee,
            metaData: state.metadata,
            transferAmount: state.transferAmount,
            feeAsset: state.feeAsset,
            currency: currency.rawValue,
        ).headerType
    }
}
