// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import Security
import Testing

struct ErrorPrimitivesTests {
    @Test
    func isAuthenticationCancelled() {
        #expect(NSError(domain: NSOSStatusErrorDomain, code: Int(errSecUserCanceled)).isAuthenticationCancelled)

        #expect(!NSError(domain: NSOSStatusErrorDomain, code: Int(errSecAuthFailed)).isAuthenticationCancelled)
        #expect(!NSError(domain: NSURLErrorDomain, code: NSURLErrorCancelled).isAuthenticationCancelled)
    }
}
