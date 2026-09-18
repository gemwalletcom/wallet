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
    func confirmCarriesTheAppConfirmInput() {
        let transfer = GemTransferData.mock()
        let value = GemDelegationDestination.confirm(transfer: transfer).navigationValue(delegation: .mock())

        #expect(value as? ConfirmTransferInput == ConfirmTransferInput(data: transfer))
    }

    @Test
    func detailsCarriesTheDelegation() {
        let delegation = Delegation.mock()

        #expect(GemDelegationDestination.details.navigationValue(delegation: delegation) as? Delegation == delegation)
    }
}
