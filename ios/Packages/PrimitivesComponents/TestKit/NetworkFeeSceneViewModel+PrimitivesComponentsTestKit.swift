// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Primitives
@testable import PrimitivesComponents
import PrimitivesTestKit

public extension NetworkFeeSceneViewModel {
    static func mock(
        chain: Chain = .ethereum,
        feeAsset: Asset? = nil,
        currency: Currency = .usd,
        selection: FeeSelection = .preset(.normal),
        rates: [FeeRate] = [],
        feeAssetPrice: Price? = nil,
        feeAmount: BigInt? = nil,
        onSelect: (@MainActor (FeeSelection) -> Void)? = nil,
    ) -> NetworkFeeSceneViewModel {
        NetworkFeeSceneViewModel(
            chain: chain,
            feeAsset: feeAsset ?? defaultFeeAsset(for: chain),
            currency: currency,
            selection: selection,
            rates: rates,
            feeAssetPrice: feeAssetPrice,
            feeAmount: feeAmount,
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
