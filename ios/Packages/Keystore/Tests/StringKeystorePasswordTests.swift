// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import Keystore
import Testing

struct StringKeystorePasswordTests {
    @Test
    func hexPasswordDecodesToBytes() throws {
        #expect(try "000102".v4KeystorePasswordBytes() == Data([0x00, 0x01, 0x02]))
        #expect(try "0x000102".v4KeystorePasswordBytes() == Data([0x00, 0x01, 0x02]))
    }

    @Test
    func nonHexPasswordFallsBackToUtf8() throws {
        let password = "not-hex-legacy-password!"
        #expect(try password.v4KeystorePasswordBytes() == Data(password.utf8))
    }

    @Test
    func oddLengthHexPasswordFallsBackToUtf8() throws {
        let password = "abc"
        #expect(try password.v4KeystorePasswordBytes() == Data(password.utf8))
    }
}
