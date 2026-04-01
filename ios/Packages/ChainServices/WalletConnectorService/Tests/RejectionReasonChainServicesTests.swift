// Copyright (c). Gem Wallet. All rights reserved.

import Testing
@testable import WalletConnectorService
import WalletConnectSign

struct RejectionReasonChainServicesTests {
    @Test
    func mapsErrors() {
        #expect(RejectionReason(from: AutoNamespacesError.requiredMethodsNotSatisfied) == .unsupportedMethods)
        #expect(RejectionReason(from: WalletConnectorServiceError.unresolvedChainId("eip155:1")) == .unsupportedChains)
    }
}
