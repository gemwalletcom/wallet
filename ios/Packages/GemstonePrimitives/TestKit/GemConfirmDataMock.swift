// Copyright (c). Gem Wallet. All rights reserved.

public import struct Gemstone.GemConfirmData
public import struct Gemstone.GemFeeOptions
public import struct Gemstone.GemFeeRate
public import enum Gemstone.GemGasPriceType
public import enum Gemstone.GemTransactionLoadMetadata
public import struct Gemstone.GemTransactionLoadFee
import Foundation

public extension GemConfirmData {
    static func mock(
        fee: GemTransactionLoadFee = .mock(),
        selectedPriority: String = "normal",
        feeRates: [GemFeeRate] = [],
        metadata: GemTransactionLoadMetadata = .none,
        scan: String? = .none,
        simulation: String? = .none,
    ) -> GemConfirmData {
        GemConfirmData(
            fee: fee,
            selectedPriority: selectedPriority,
            feeRates: feeRates,
            metadata: metadata,
            scan: scan,
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
