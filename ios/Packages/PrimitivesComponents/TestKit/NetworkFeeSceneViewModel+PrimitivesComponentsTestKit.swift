// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Primitives
@testable import PrimitivesComponents
import PrimitivesTestKit

public extension NetworkFeeSceneViewModel {
    static func mock(
        chain: Chain = .ethereum,
        feeAsset: Asset? = nil,
        priority: FeePriority = .normal,
        currency: Currency = .usd,
        feeAmount: BigInt? = nil,
        allowsCustomFee: Bool = false,
    ) -> NetworkFeeSceneViewModel {
        let feeAsset = feeAsset ?? defaultFeeAsset(for: chain)
        return NetworkFeeSceneViewModel(
            chain: chain,
            feeAsset: feeAsset,
            priority: priority,
            currency: currency,
            feeAmount: feeAmount,
            allowsCustomFee: allowsCustomFee,
        )
    }
}

public extension NetworkFeeCustomViewModel {
    static func mock(
        chain: Chain = .bitcoin,
        feeAsset: Asset? = nil,
        feeAssetPrice: Price? = nil,
        currency: Currency = .usd,
        baseFee: BigInt? = 1000,
        baseTotal: BigInt? = 2,
        initialRate: BigInt? = nil,
        onSelect: @escaping (BigInt) -> Void = { _ in },
    ) -> NetworkFeeCustomViewModel {
        NetworkFeeCustomViewModel(
            chain: chain,
            feeAsset: feeAsset ?? .mock(id: AssetId(chain: .bitcoin, tokenId: .none)),
            feeAssetPrice: feeAssetPrice,
            currency: currency,
            baseFee: baseFee,
            baseTotal: baseTotal,
            initialRate: initialRate,
            onSelect: onSelect,
        )
    }
}

private extension NetworkFeeSceneViewModel {
    static func defaultFeeAsset(for chain: Chain) -> Asset {
        switch chain {
        case .ethereum:
            .mockEthereum()
        case .solana:
            .mock(
                id: AssetId(chain: .solana, tokenId: .none),
                name: "Solana",
                symbol: "SOL",
                decimals: 9,
                type: .native,
            )
        case .hyperCore:
            .mockHypercore()
        default:
            .mock(id: AssetId(chain: chain, tokenId: .none))
        }
    }
}
