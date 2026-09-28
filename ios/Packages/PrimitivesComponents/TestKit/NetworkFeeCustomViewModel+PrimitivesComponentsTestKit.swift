// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import struct Gemstone.GemCustomFeeSession
import struct Gemstone.GemFeeRateRows
import GemstonePrimitives
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesComponents
import PrimitivesTestKit

public extension NetworkFeeCustomViewModel {
    static func mock(
        feeAsset: Asset = .mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18),
        unitType: FeeUnitType = .gwei,
        decimals: Int = 9,
        baseFee: BigInt? = BigInt(21000),
        baseTotal: BigInt? = BigInt(1_000_000_000),
        normalTotal: BigInt? = BigInt(2_000_000_000),
        onSelect: @escaping @MainActor (BigInt) -> Void = { _ in },
    ) -> NetworkFeeCustomViewModel {
        NetworkFeeCustomViewModel(
            session: GemCustomFeeSession(
                feeAsset: feeAsset.toGem(),
                input: "",
                format: NumberInput.format(),
                rows: .mock(showsOptions: true, unitType: unitType.toGem(), unitDecimals: UInt32(decimals), selectedTotal: baseTotal, normalTotal: normalTotal),
                loadedFee: baseFee,
                price: nil,
                currency: Currency.usd.toGem(),
            ),
            onSelect: onSelect,
        )
    }
}
