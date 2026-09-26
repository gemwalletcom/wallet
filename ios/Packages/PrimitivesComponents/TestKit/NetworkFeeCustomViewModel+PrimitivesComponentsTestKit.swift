// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import struct Gemstone.GemFeeRateRows
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import PrimitivesTestKit

public extension NetworkFeeCustomViewModel {
    static func mock(
        feeAsset: Asset = .mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18),
        unitType: FeeUnitType = .gwei,
        decimals: Int = 9,
        initialRate: BigInt? = nil,
        baseFee: BigInt? = BigInt(21000),
        baseTotal: BigInt? = BigInt(1_000_000_000),
        normalTotal: BigInt? = BigInt(2_000_000_000),
        onSelect: @escaping @MainActor (BigInt) -> Void = { _ in },
    ) -> NetworkFeeCustomViewModel {
        NetworkFeeCustomViewModel(
            feeAsset: feeAsset,
            rows: GemFeeRateRows(
                rows: [],
                showsOptions: true,
                unitType: unitType.toGem(),
                unitDecimals: UInt32(decimals),
                selectedTotal: baseTotal,
                normalTotal: normalTotal,
            ),
            baseFee: baseFee,
            initialRate: initialRate,
            price: nil,
            currency: .usd,
            onSelect: onSelect,
        )
    }
}
