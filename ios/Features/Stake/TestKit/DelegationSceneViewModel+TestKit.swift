// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import protocol Gemstone.GemStakeServiceProtocol
import GemstonePrimitivesTestKit
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
        stakeService: any GemStakeServiceProtocol = GemStakeServiceMock(),
        onNavigate: StakeRouteAction = nil,
        onSelectAddress: (@MainActor @Sendable (ChainAddress) -> Void)? = nil,
    ) -> DelegationSceneViewModel {
        let validator = DelegationValidator.mock(chain, providerType: providerType)
        let base = DelegationBase.mock(state: state, assetId: .mock(chain: chain), rewards: rewards)
        let delegation = Delegation.mock(state: state, validator: validator, base: base)
        return DelegationSceneViewModel(
            wallet: wallet,
            delegation: delegation,
            asset: chain.asset,
            service: stakeService,
            onNavigate: onNavigate,
            onSelectAddress: onSelectAddress,
        )
    }
}
