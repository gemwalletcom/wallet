// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemWalletConnectError
import enum Gemstone.GemWalletConnectRejectionReason
import Testing
@testable import WalletConnectorService
import WalletConnectSign

struct RejectionReasonWalletConnectorServiceTests {
    @Test
    func everyErrorNamesACoreRejectionReason() {
        #expect(GemWalletConnectRejectionReason(from: AutoNamespacesError.requiredMethodsNotSatisfied) == .unsupportedMethods)
        #expect(GemWalletConnectRejectionReason(from: AutoNamespacesError.requiredEventsNotSatisfied) == .unsupportedEvents)
        #expect(GemWalletConnectRejectionReason(from: GemWalletConnectError.UnsupportedChains) == .unsupportedChains)
        #expect(GemWalletConnectRejectionReason(from: GemWalletConnectError.UnsupportedWallets) == .unsupportedAccounts)
        #expect(GemWalletConnectRejectionReason(from: GemWalletConnectError.InvalidOrigin) == .userRejected)
    }

    @Test
    func theCoreReasonPicksTheSdkReason() {
        #expect(RejectionReason(.unsupportedChains) == .unsupportedChains)
        #expect(RejectionReason(.unsupportedAccounts) == .unsupportedAccounts)
        #expect(RejectionReason(.unsupportedEvents) == .unsupportedEvents)
        #expect(RejectionReason(.unsupportedMethods) == .unsupportedMethods)
        #expect(RejectionReason(.userRejected) == .userRejected)
    }
}
