// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import Primitives
import PrimitivesTestKit
import Testing
@testable import Transfer
import TransferTestKit

struct AmountEarnViewModelTests {
    @Test
    func aDepositNamesTheValidatorAndAWithdrawTheOneItIsLeaving() {
        let validator = DelegationValidator.mock(name: "Figment")
        let leaving = DelegationValidator.mock(name: "Chorus One")
        let deposit = AmountEarnViewModel.mock(action: .deposit(validator.toGem()))
        let withdraw = AmountEarnViewModel.mock(action: .withdraw(Delegation.mock(validator: leaving).toGem()))

        #expect(deposit.providerRow?.name == "Figment")
        #expect(withdraw.providerRow?.name == "Chorus One")
        #expect(deposit.title != withdraw.title)
    }
}
