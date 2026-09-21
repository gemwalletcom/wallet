// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import struct Gemstone.GemFeeRateRows
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import PrimitivesTestKit

public extension NetworkFeeCustomViewModel {
    static func mock(
        chain: Chain = .ethereum,
        feeAsset: Asset = .mockEthereum(),
        unitType: FeeUnitType = .gwei,
        decimals: Int = 9,
        initialRate: BigInt? = nil,
        baseFee: BigInt? = BigInt(21000),
        baseTotal: BigInt? = BigInt(1_000_000_000),
        normalTotal: BigInt? = BigInt(2_000_000_000),
        onSelect: @escaping @MainActor (BigInt) -> Void = { _ in },
    ) -> NetworkFeeCustomViewModel {
        NetworkFeeCustomViewModel(
            chain: chain,
            feeAsset: feeAsset,
            rows: GemFeeRateRows(
                rows: [],
                showsOptions: true,
                unitType: unitType.toGem(),
                unitDecimals: UInt32(decimals),
                supportsCustomFee: true,
                selectedTotal: baseTotal,
                normalTotal: normalTotal,
                customRate: nil,
            ),
            baseFee: baseFee,
            initialRate: initialRate,
            onSelect: onSelect,
            display: { AmountDisplay.numeric(asset: feeAsset, price: nil, value: $0, currency: Currency.usd.rawValue, formatter: .auto) },
        )
    }
}
