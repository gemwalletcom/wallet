// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemServiceError
@testable import GemstoneServices
import GemstoneServicesTestKit
import Testing

struct GemstoneKeystorePasswordTests {
    @Test
    func passwordAndAuthenticationComeFromThePasswordOwner() throws {
        let keystorePassword = GemstoneKeystorePassword(password: MockKeystorePassword(memoryPassword: "stored", availableAuthentication: .biometrics))

        #expect(try keystorePassword.getPassword(createIfMissing: false) == "stored")
        #expect(try keystorePassword.authentication() == .biometrics)
    }

    @Test
    func cancelledAuthenticationIsReportedAsCancelled() {
        let password = MockKeystorePassword()
        password.getAuthenticationError = NSError(domain: NSOSStatusErrorDomain, code: Int(errSecUserCanceled))
        let keystorePassword = GemstoneKeystorePassword(password: password)

        #expect(throws: GemServiceError.Cancelled) { try keystorePassword.getPassword(createIfMissing: true) }
    }
}
