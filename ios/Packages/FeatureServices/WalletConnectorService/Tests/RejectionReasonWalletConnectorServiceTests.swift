// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemWalletConnectError
import enum Gemstone.GemWalletConnectRejectionReason
import Primitives
import Testing
@testable import WalletConnectorService

struct RejectionReasonWalletConnectorServiceTests {
    @Test
    func everyErrorNamesACoreRejectionReason() {
        #expect(GemWalletConnectRejectionReason(from: GemWalletConnectError.UnsupportedChains) == .unsupportedChains)
        #expect(GemWalletConnectRejectionReason(from: GemWalletConnectError.UnsupportedWallets) == .unsupportedAccounts)
        #expect(GemWalletConnectRejectionReason(from: GemWalletConnectError.InvalidOrigin) == .userRejected)
        #expect(GemWalletConnectRejectionReason(from: AnyError("cancelled")) == .userRejected)
    }
}
