// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import enum Gemstone.GasPriceType
import enum Gemstone.GemConfirmFeeSelection
import GemstonePrimitives
import GemstonePrimitivesTestKit
import Primitives
@testable import PrimitivesComponents
import PrimitivesTestKit

public extension NetworkFeeCustomViewModel {
    @MainActor
    static func mock(
        feeAsset: Asset = .mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18),
        unitType: FeeUnitType = .gwei,
        decimals: Int = 9,
        loadedFee: BigInt? = BigInt(21000),
        selected: GasPriceType? = .regular(gasPrice: BigInt(1_000_000_000)),
        normal: GasPriceType? = .regular(gasPrice: BigInt(2_000_000_000)),
        networkBaseFee: BigInt? = nil,
        onSelect: @escaping @MainActor (GemConfirmFeeSelection) -> Void = { _ in },
    ) -> NetworkFeeCustomViewModel {
        NetworkFeeSceneViewModel.mock(
            feeAsset: feeAsset,
            feeRates: .mock(showsOptions: true, unitType: unitType.toGem(), unitDecimals: UInt32(decimals), selected: selected, normal: normal, baseFee: networkBaseFee),
            feeAmount: loadedFee,
            onSelect: onSelect,
        ).customFeeModel()!
    }
}
