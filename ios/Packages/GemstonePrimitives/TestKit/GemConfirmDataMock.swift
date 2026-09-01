// Copyright (c). Gem Wallet. All rights reserved.

public import enum Gemstone.FeePriority
public import struct Gemstone.GemConfirmData
public import struct Gemstone.GemConfirmPreload
public import struct Gemstone.GemConfirmMetadata
public import struct Gemstone.GemAssetBalance
public import enum Gemstone.GemTransferAmountResult
public import struct Gemstone.GemTransferAmount
public import struct Gemstone.GemConfirmInput
public import struct Gemstone.GemFeeOptions
public import struct Gemstone.GemFeeRate
public import enum Gemstone.GemGasPriceType
public import enum Gemstone.GemTransactionLoadMetadata
public import struct Gemstone.GemTransactionLoadFee
import Foundation
import GemstonePrimitivesTestKit
import GemstonePrimitives
import Primitives
import PrimitivesTestKit

public extension GemConfirmData {
    static func mock(
        input: GemConfirmInput = TransferData.mock().confirmInput(from: .mock()),
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
        fee: String = "1",
        gasPriceType: GemGasPriceType = .regular(gasPrice: "1"),
        gasLimit: String = "1",
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
        metadata: GemConfirmMetadata? = nil,
        feeAsset: Primitives.Asset = .mockEthereum(),
        amount: GemTransferAmountResult = .amount(amount: GemTransferAmount(value: "1", networkFee: "1", isMaxAmount: false)),
    ) -> GemConfirmPreload {
        GemConfirmPreload(
            confirmData: confirmData,
            metadata: metadata ?? GemConfirmMetadata(
                assetBalance: .mock(assetId: feeAsset.id.identifier),
                feeAssetBalance: .mock(assetId: feeAsset.id.identifier),
                prices: [],
            ),
            feeAsset: feeAsset.map(),
            amount: amount,
        )
    }
}

public extension GemAssetBalance {
    static func mock(assetId: String) -> GemAssetBalance {
        GemAssetBalance(
            assetId: assetId,
            available: "0",
            frozen: "0",
            locked: "0",
            staked: "0",
            pending: "0",
            pendingUnconfirmed: "0",
            rewards: "0",
            reserved: "0",
            withdrawable: "0",
            earn: "0",
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
