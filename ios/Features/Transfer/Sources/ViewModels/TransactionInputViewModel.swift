// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemConfirmMetadata
import GemstoneServices
import BigInt
import Foundation
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import struct Gemstone.GemTransferData

public struct TransactionInputViewModel: Sendable {
    let data: GemTransferData
    let fee: Fee?
    let metaData: GemConfirmMetadata?
    let transferAmount: TransferAmountValidation?
    let feeAsset: Asset

    private let currency: String

    public init(
        data: GemTransferData,
        fee: Fee?,
        metaData: GemConfirmMetadata?,
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
        case .failure, .none: BigInt(core: data.value)
        }
    }

    var asset: Asset {
        switch data.inputType {
        case let .perpetual(_, type): Primitives.PerpetualType(core: type).baseAsset
        default: data.inputType.asset
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
        switch data.inputType {
        case .withdrawal: PerpetualConfig.depositAsset
        default: data.inputType.asset
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
            dataType: data.inputType,
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
