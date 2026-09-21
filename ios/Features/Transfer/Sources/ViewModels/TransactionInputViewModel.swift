// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import struct Gemstone.GemConfirmMetadata
import class Gemstone.GemPerpetual
import struct Gemstone.GemTransactionLoadFee
import struct Gemstone.GemTransferData
import GemstonePrimitives
import GemstoneServices
import Primitives
import PrimitivesComponents
import Style

public struct TransactionInputViewModel: Sendable {
    let data: GemTransferData
    let fee: GemTransactionLoadFee?
    let metaData: GemConfirmMetadata?
    let transferAmount: TransferAmountValidation?
    let feeAsset: Asset

    private let currency: String

    public init(
        data: GemTransferData,
        fee: GemTransactionLoadFee?,
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
        case .failure, .none: data.value
        }
    }

    var asset: Asset {
        switch data.inputType {
        case let .perpetual(_, type): type.baseAsset
        default: data.asset
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
            sign: .none,
        )
    }

    private var displayAsset: Asset {
        switch data.inputType {
        case .withdrawal: GemPerpetual(provider: .hypercore).depositAsset().toPrimitives()
        case .transfer,
             .deposit,
             .swap,
             .stake,
             .tokenApprove,
             .generic,
             .payment,
             .transferNft,
             .account,
             .perpetual,
             .earn: data.asset
        }
    }

    var headerType: TransactionHeaderType {
        TransactionHeaderTypeBuilder.build(
            infoModel: infoModel,
            transfer: data,
            metadata: metaData,
        )
    }
}
