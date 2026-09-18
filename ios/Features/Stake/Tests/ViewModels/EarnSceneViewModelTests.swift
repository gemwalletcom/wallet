// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesTestKit
@testable import Stake
import StakeTestKit
@testable import Store
import Testing

@MainActor
struct EarnSceneViewModelTests {
    @Test
    func depositNavigatesToTheAmountOfTheFirstProvider() {
        let provider = DelegationValidator.mock(.ethereum, providerType: .earn)
        var route: StakeRoute?
        let model = EarnSceneViewModel.mock(
            stakeService: GemStakeServiceMock(validators: [provider.toGem()]),
            onNavigate: { route = $0 },
        )
        model.providersQuery.value = [provider]

        #expect(model.canDeposit)
        model.onSelectDeposit()

        guard case let .transfer(.amount(input)) = route, case .earn = input.type else {
            Issue.record("expected an earn amount route, got \(String(describing: route))")
            return
        }
    }

    @Test
    func noProviderMeansNoDeposit() {
        var route: StakeRoute?
        let model = EarnSceneViewModel.mock(onNavigate: { route = $0 })

        #expect(model.canDeposit == false)
        model.onSelectDeposit()

        #expect(route == nil)
    }
}
