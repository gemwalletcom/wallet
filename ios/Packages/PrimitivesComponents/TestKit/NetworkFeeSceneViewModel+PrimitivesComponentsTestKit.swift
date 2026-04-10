// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Primitives
import PrimitivesTestKit
@testable import PrimitivesComponents

public extension NetworkFeeSceneViewModel {
    static func mock(
        chain: Chain = .ethereum,
        feeAsset: Asset? = nil,
        priority: FeePriority = .normal,
        currency: Currency = .usd,
        feeAmount: BigInt? = nil,
    ) -> NetworkFeeSceneViewModel {
        let feeAsset = feeAsset ?? defaultFeeAsset(for: chain)
        return NetworkFeeSceneViewModel(
            chain: chain,
            feeAsset: feeAsset,
            priority: priority,
            currency: currency,
            feeAmount: feeAmount,
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
