// Copyright (c). Gem Wallet. All rights reserved.

public import BigInt
public import struct Gemstone.AssetPrice
public import enum Gemstone.FeePriority
public import enum Gemstone.GasPriceType
public import struct Gemstone.GemAssetBalance
public import struct Gemstone.GemConfirmFee
public import struct Gemstone.GemConfirmLoad
public import struct Gemstone.GemConfirmMetadata
public import struct Gemstone.GemConfirmSimulation
public import struct Gemstone.GemConfirmSimulationState
public import struct Gemstone.GemFeeAsset
public import struct Gemstone.GemFeeOptionItem
public import enum Gemstone.GemListRow
public import struct Gemstone.GemTransferAmount
public import enum Gemstone.GemTransferAmountResult
public import struct Gemstone.TransferDataExtra
import Foundation
import struct Gemstone.ApprovalData
import func Gemstone.feeAmount
import struct Gemstone.GemTransferData
import GemstonePrimitives
import Primitives
import PrimitivesTestKit

public extension TransferDataExtra {
    static func mock(
        to: String = "",
        gasLimit: BigInt? = .none,
        gasPrice: GasPriceType? = .none,
        data: Data? = .none,
        outputType: Primitives.TransferDataOutputType = .encodedTransaction,
        outputAction: Primitives.TransferDataOutputAction = .send,
        transactionType: Primitives.TransactionType = .transfer,
        approval: Gemstone.ApprovalData? = .none,
    ) -> TransferDataExtra {
        TransferDataExtra(
            to: to,
            gasLimit: gasLimit,
            gasPrice: gasPrice,
            data: data,
            outputType: outputType.toGem(),
            outputAction: outputAction.toGem(),
            transactionType: transactionType.toGem(),
            approval: approval,
        )
    }
}
