// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemConfirmInput
public import BigInt
public import enum Gemstone.FeePriority
public import struct Gemstone.GemConfirmData
public import struct Gemstone.GemConfirmPreload
public import struct Gemstone.GemConfirmLoad
public import struct Gemstone.GemConfirmSimulation
public import struct Gemstone.GemConfirmSimulationState
public import struct Gemstone.GemFeeAsset
public import struct Gemstone.GemConfirmMetadata
public import struct Gemstone.GemAssetBalance
public import enum Gemstone.GemTransferAmountResult
public import struct Gemstone.GemTransferAmount
public import struct Gemstone.GemConfirmInput
public import struct Gemstone.GemFeeOptions
public import struct Gemstone.GemFeeRate
public import enum Gemstone.GasPriceType
public import struct Gemstone.TransferDataExtra
public import enum Gemstone.GemTransactionLoadMetadata
public import struct Gemstone.GemTransactionLoadFee
import Foundation
import GemstonePrimitivesTestKit
import GemstonePrimitives
import Primitives
import PrimitivesTestKit
import struct Gemstone.GemTransferData

public extension GemConfirmData {
    static func mock(
        input: GemConfirmInput = GemConfirmInput(from: Primitives.Account.mock().map(), transfer: GemTransferData.mock()),
        fee: GemTransactionLoadFee = .mock(),
        selectedPriority: Gemstone.FeePriority = .normal,
        feeRates: [GemFeeRate] = [],
        metadata: GemTransactionLoadMetadata = .none,
        simulation: String? = .none,
    ) -> GemConfirmData {
        GemConfirmData(
            input: input,
            fee: fee,
            selectedPriority: selectedPriority,
            feeRates: feeRates,
            metadata: metadata,
            simulation: simulation,
        )
    }
}

public extension GemTransactionLoadFee {
    static func mock(
        fee: BigInt = 1,
        gasPriceType: GasPriceType = .regular(gasPrice: 1),
        gasLimit: BigInt = 1,
        options: GemFeeOptions = GemFeeOptions(options: [:]),
        feeAsset: String = "bitcoin",
    ) -> GemTransactionLoadFee {
        GemTransactionLoadFee(
            fee: fee,
            gasPriceType: gasPriceType,
            gasLimit: gasLimit,
            options: options,
            feeAsset: feeAsset,
        )
    }
}

public extension GemConfirmPreload {
    static func mock(
        confirmData: GemConfirmData = .mock(),
        amount: GemTransferAmountResult = .amount(amount: GemTransferAmount(value: 1, networkFee: 1, isMaxAmount: false)),
    ) -> GemConfirmPreload {
        GemConfirmPreload(confirmData: confirmData, amount: amount)
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
        )
    }
}

public extension GemConfirmMetadata {
    static func mock(assetId: String = Primitives.Asset.mock().id.identifier) -> GemConfirmMetadata {
        GemConfirmMetadata(
            assetBalance: .mock(assetId: assetId),
            feeAssetBalance: .mock(assetId: assetId),
            prices: [],
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
        approval: String? = .none,
    ) -> TransferDataExtra {
        TransferDataExtra(
            to: to,
            gasLimit: gasLimit,
            gasPrice: gasPrice,
            data: data,
            outputType: outputType.map(),
            outputAction: outputAction.map(),
            transactionType: transactionType.map(),
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
        sender: Primitives.Account = .mock(),
        feeAsset: Primitives.Asset = .mockEthereum(),
        metadata: GemConfirmMetadata = .mock(),
        feeAssets: [GemFeeAsset] = [],
        simulation: GemConfirmSimulation? = nil,
        warnings: [Primitives.SimulationWarning] = [],
        addressName: Primitives.AddressName? = nil,
        preload: GemConfirmPreload? = .mock(),
    ) -> GemConfirmLoad {
        GemConfirmLoad(
            sender: sender.map(),
            feeAsset: feeAsset.map(),
            metadata: metadata,
            feeAssets: feeAssets,
            simulation: GemConfirmSimulationState(chain: Primitives.Chain.ethereum.rawValue, result: nil, warnings: warnings.map { $0.json() }, simulation: simulation, addressNames: []),
            addressName: addressName?.map(),
            preload: preload,
        )
    }
}
