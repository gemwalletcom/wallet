// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import GemstonePrimitivesTestKit
import protocol Gemstone.GemStakeServiceProtocol
import Primitives
import PrimitivesTestKit
@testable import Stake

public extension DelegationSceneViewModel {
    static func mock(
        wallet: Wallet = .mock(),
        chain: Chain = .cosmos,
        state: DelegationState = .active,
        rewards: BigInt = .zero,
        providerType: StakeProviderType = .stake,
        validators: [DelegationValidator] = [],
        stakeService: any GemStakeServiceProtocol = GemStakeServiceMock(),
    ) -> DelegationSceneViewModel {
        let validator = DelegationValidator.mock(chain, providerType: providerType)
        let base = DelegationBase.mock(state: state, assetId: .mock(chain), rewards: rewards)
        let delegation = Delegation.mock(state: state, validator: validator, base: base)
        return DelegationSceneViewModel(
            wallet: wallet,
            model: DelegationViewModel(service: stakeService, delegation: delegation, asset: chain.asset, currencyCode: "USD"),
            asset: chain.asset,
            service: stakeService,
            validators: validators,
            onAmountInputAction: nil,
            onTransferAction: nil,
        )
    }
}
