// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemErrorText
import enum Gemstone.GemServiceError
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
}
