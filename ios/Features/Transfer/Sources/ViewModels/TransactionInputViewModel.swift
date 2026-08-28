// Copyright (c). Gem Wallet. All rights reserved.

import GemstoneServices
import BigInt
import Foundation
import GemstonePrimitives
import Primitives
import PrimitivesComponents

public struct TransactionInputViewModel: Sendable {
    let data: TransferData
    let fee: Fee?
    let metaData: TransferDataMetadata?
    let transferAmount: TransferAmountValidation?
    let feeAsset: Asset

    private let currency: String

    public init(
        data: TransferData,
        fee: Fee?,
        metaData: TransferDataMetadata?,
        transferAmount: TransferAmountValidation?,
        feeAsset: Asset,
        currency: String,
    ) {
        self.fee = fee
        self.data = data
        self.metaData = metaData
        self.transferAmount = transferAmount
        self.feeAsset = feeAsset
        self.currency = currency
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
        TransactionInfoViewModel(
            currency: currency,
            asset: displayAsset,
            assetPrice: metaData?.assetPrice,
            feeAsset: feeAsset,
            feeAssetPrice: metaData?.feePrice,
            value: value,
            feeValue: fee?.fee,
            direction: nil,
        )
    }

    private var displayAsset: Asset {
        switch data.type {
        case .withdrawal: PerpetualConfig.depositAsset
        default: data.type.asset
        }
    }

    var networkFeeText: String? {
        infoModel.feeDisplay?.amount.text ?? "-"
    }

    var networkFeeFiatText: String? {
        infoModel.feeDisplay?.fiat?.text
    }

    var networkFeeAmount: BigInt? {
        fee?.fee
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
