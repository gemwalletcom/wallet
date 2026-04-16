// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
@testable import Stake

public extension DelegationSceneViewModel {
    static func mock(
        wallet: Wallet = .mock(),
        chain: Chain = .cosmos,
        state: DelegationState = .active,
        rewards: String = .empty,
        providerType: StakeProviderType = .stake,
        validators: [DelegationValidator] = [],
    ) -> DelegationSceneViewModel {
        let validator = DelegationValidator.mock(chain, providerType: providerType)
        let base = DelegationBase.mock(state: state, assetId: .mock(chain), rewards: rewards)
        let delegation = Delegation.mock(state: state, validator: validator, base: base)
        return DelegationSceneViewModel(
            wallet: wallet,
            model: DelegationViewModel(delegation: delegation, asset: chain.asset, currencyCode: "USD"),
            asset: chain.asset,
            validators: validators,
            onAmountInputAction: nil,
            onTransferAction: nil,
        )
    }
}
