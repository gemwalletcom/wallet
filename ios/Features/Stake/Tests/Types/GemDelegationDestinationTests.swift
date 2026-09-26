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
        let route = GemDelegationDestination.confirm(transfer: transfer).route(delegation: .mock())

        #expect(route == .transfer(.confirm(transfer)))
    }

    @Test
    func detailsCarriesTheDelegation() {
        let delegation = Delegation.mock()
        let route = GemDelegationDestination.details.route(delegation: delegation)

        #expect(route == .delegation(delegation))
    }
}
