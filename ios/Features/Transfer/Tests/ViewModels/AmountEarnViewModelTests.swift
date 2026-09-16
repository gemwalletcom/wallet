// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemAmountService
import GemstonePrimitives
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
import Testing
@testable import Transfer

struct AmountEarnViewModelTests {
    @Test
    func aDepositNamesTheValidatorAndAWithdrawTheOneItIsLeaving() {
        let validator = DelegationValidator.mock(name: "Figment")
        let leaving = DelegationValidator.mock(name: "Chorus One")
        let deposit = AmountEarnViewModel(asset: .mock(), action: .deposit(validator.toGem()), service: GemAmountService.mock())
        let withdraw = AmountEarnViewModel(
            asset: .mock(),
            action: .withdraw(Delegation.mock(validator: leaving).toGem()),
            service: GemAmountService.mock(),
        )

        #expect(deposit.provider.name == "Figment")
        #expect(deposit.providerRow.name == "Figment")
        #expect(withdraw.provider.name == "Chorus One")
        #expect(deposit.title != withdraw.title)
    }
}
