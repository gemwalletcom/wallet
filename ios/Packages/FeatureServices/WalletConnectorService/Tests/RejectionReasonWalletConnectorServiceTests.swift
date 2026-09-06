// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemWalletConnectError
import Testing
@testable import WalletConnectorService
import WalletConnectSign

struct RejectionReasonWalletConnectorServiceTests {
    @Test
    func mapsErrors() {
        #expect(RejectionReason(from: AutoNamespacesError.requiredMethodsNotSatisfied) == .unsupportedMethods)
        #expect(RejectionReason(from: GemWalletConnectError.UnsupportedChains) == .unsupportedChains)
        #expect(RejectionReason(from: GemWalletConnectError.InvalidOrigin) == .userRejected)
    }
}
