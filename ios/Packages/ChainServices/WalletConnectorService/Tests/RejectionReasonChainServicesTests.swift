// Copyright (c). Gem Wallet. All rights reserved.

import Testing
import WalletConnectSign
@testable import WalletConnectorService

struct RejectionReasonChainServicesTests {

    @Test
    func mapsErrors() {
        #expect(RejectionReason(from: AutoNamespacesError.requiredMethodsNotSatisfied) == .unsupportedMethods)
        #expect(RejectionReason(from: WalletConnectorServiceError.unresolvedChainId("eip155:1")) == .unsupportedChains)
    }
}
