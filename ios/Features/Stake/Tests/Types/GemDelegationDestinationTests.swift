// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemDelegationDestination
import struct Gemstone.GemTransferData
import GemstonePrimitives
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesTestKit
@testable import Stake
import Testing

struct GemDelegationDestinationTests {
    @Test
    func confirmBecomesATransferRoute() {
        let transfer = GemTransferData.mock()
        let route = GemDelegationDestination.confirm(transfer: transfer).route(delegation: .mock(), validators: [])

        #expect(route == .transfer(.confirm(transfer)))
    }

    @Test
    func detailsCarriesTheDelegationAndItsValidators() {
        let delegation = Delegation.mock()
        let validators = [DelegationValidator.mock()]
        let route = GemDelegationDestination.details.route(delegation: delegation, validators: validators)

        #expect(route == .delegation(DelegationInput(delegation: delegation, validators: validators)))
    }
}
