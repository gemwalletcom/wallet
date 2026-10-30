// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemErrorText
import enum Gemstone.GemServiceError
import enum Gemstone.GemWalletConnectFailure
import Localization
@testable import PrimitivesComponents
import Testing

struct ErrorExtensionTests {
    @Test
    func networkErrorDescription() {
        let error = NSError(domain: NSURLErrorDomain, code: NSURLErrorNotConnectedToInternet)
        #expect(error.networkOrNoDataDescription == error.localizedDescription)
    }

    @Test
    func nonNetworkErrorDescription() {
        let error = NSError(domain: "TestDomain", code: 500)
        #expect(error.networkOrNoDataDescription == "No data available")
    }

    @Test
    func coreOfflineErrorDescription() {
        #expect(GemServiceError.Offline.networkOrNoDataDescription == GemErrorText.networkOffline.text)
        #expect(GemServiceError.Api(msg: "Price not found").networkOrNoDataDescription == "No data available")
    }

    @Test
    func walletConnectFailureDescription() {
        let errors: [any Error] = [GemWalletConnectFailure.maliciousOrigin, GemWalletConnectFailure.expired, GemWalletConnectFailure.failed(error: .networkOffline)]
        #expect(errors.map(\.localizedDescription) == [
            Localized.Errors.Connections.maliciousOrigin,
            Localized.WalletConnect.requestExpired,
            GemErrorText.networkOffline.text,
        ])
    }
}
