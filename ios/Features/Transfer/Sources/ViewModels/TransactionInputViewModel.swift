// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Blockchain
import Foundation
import GemstonePrimitives
import Preferences
import Primitives
import PrimitivesComponents

public struct TransactionInputViewModel: Sendable {
    let data: TransferData
    let transactionData: TransactionData?
    let metaData: TransferDataMetadata?
    let transferAmount: TransferAmountValidation?

    private let preferences: Preferences

    public init(
        data: TransferData,
        transactionData: TransactionData?,
        metaData: TransferDataMetadata?,
        transferAmount: TransferAmountValidation?,
        preferences: Preferences = Preferences.standard,
    ) {
        self.transactionData = transactionData
        self.data = data
        self.metaData = metaData
        self.transferAmount = transferAmount
        self.preferences = preferences
    }

    var value: BigInt {
        switch transferAmount {
        case let .success(amount): amount.value
        case .failure, .none: data.value
        }
    }

    var asset: Asset {
        switch data.type {
        case let .perpetual(_, type): type.baseAsset
        default: data.type.asset
        }
    }

    var infoModel: TransactionInfoViewModel {
        let asset = data.type.asset

        return TransactionInfoViewModel(
            currency: preferences.currency,
            asset: asset,
            assetPrice: metaData?.assetPrice,
            feeAsset: asset.feeAsset,
            feeAssetPrice: metaData?.feePrice,
            value: value,
            feeValue: transactionData?.fee.fee,
            direction: nil,
        )
    }

    var networkFeeText: String? {
        infoModel.feeDisplay?.amount.text ?? "-"
    }

    var networkFeeFiatText: String? {
        infoModel.feeDisplay?.fiat?.text
    }

    var networkFeeAmount: BigInt? {
        transactionData?.fee.fee
    }

    var headerType: TransactionHeaderType {
        TransactionHeaderTypeBuilder.build(
            infoModel: infoModel,
            dataType: data.type,
            metadata: metaData,
        )
    }

    var isReady: Bool {
        if case .success = transferAmount {
            return true
        }
        return false
    }
}
