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

public extension GemConfirmFee {
    static func mock(
        value: BigInt = 1,
        additionalFees: [GemFeeOptionItem] = [],
        selectedPriority: Gemstone.FeePriority = .normal,
        amount: GemTransferAmountResult = .amount(amount: GemTransferAmount(value: 1, networkFee: 1, isMaxAmount: false)),
    ) -> GemConfirmFee {
        GemConfirmFee(
            value: value,
            formatted: Gemstone.feeAmount(asset: Asset.mockEthereum().toGem(), value: value, price: nil, currency: Primitives.Currency.usd.toGem()),
            additionalFees: additionalFees,
            selectedPriority: selectedPriority,
            amount: amount,
        )
    }
}

public extension GemAssetBalance {
    static func mock(assetId: String) -> GemAssetBalance {
        GemAssetBalance(
            assetId: assetId,
            available: 0,
            frozen: 0,
            locked: 0,
            staked: 0,
            pending: 0,
            pendingUnconfirmed: 0,
            rewards: 0,
            reserved: 0,
            withdrawable: 0,
            earn: 0,
            metadata: nil,
            isActive: true,
        )
    }
}

public extension GemConfirmMetadata {
    static func mock(
        assetId: String = Primitives.Asset.mock().id.identifier,
        prices: [Gemstone.AssetPrice] = [],
    ) -> GemConfirmMetadata {
        GemConfirmMetadata(
            assetBalance: .mock(assetId: assetId),
            feeAssetBalance: .mock(assetId: assetId),
            prices: prices,
        )
    }
}

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

public extension GemTransferAmount {
    static func mock(
        value: BigInt = 100,
        networkFee: BigInt = 1,
        isMaxAmount: Bool = false,
    ) -> GemTransferAmount {
        GemTransferAmount(value: value, networkFee: networkFee, isMaxAmount: isMaxAmount)
    }
}

public extension GemConfirmLoad {
    static func mock(
        transfer: GemTransferData = .mock(),
        sender: Primitives.Account = .mock(),
        feeAsset: Primitives.Asset = .mockEthereum(),
        metadata: GemConfirmMetadata = .mock(),
        feeAssets: [GemFeeAsset] = [],
        simulation: GemConfirmSimulation? = nil,
        warnings: [GemListRow] = [],
        addressName: Primitives.AddressName? = nil,
        fee: GemConfirmFee? = .mock(),
    ) -> GemConfirmLoad {
        GemConfirmLoad(
            transfer: transfer,
            sender: sender.toGem(),
            feeAsset: feeAsset.toGem(),
            metadata: metadata,
            feeAssets: feeAssets,
            simulation: GemConfirmSimulationState(chain: Primitives.Chain.ethereum.rawValue, warnings: warnings, simulation: simulation),
            addressName: addressName?.toGem(),
            fee: fee,
        )
    }
}
