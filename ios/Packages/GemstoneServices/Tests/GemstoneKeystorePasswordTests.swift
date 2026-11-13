// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemServiceError
@testable import GemstoneServices
import GemstoneServicesTestKit
import LocalAuthentication
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

    @Test
    func readsThePasswordWithoutAuthentication() throws {
        let password = MockKeystorePassword(memoryPassword: "stored", availableAuthentication: .none)
        let keystorePassword = GemstoneKeystorePassword(password: password, unlockContext: UnlockContextStore())

        #expect(try keystorePassword.getPassword(createIfMissing: false) == "stored")
        #expect(password.passwordContext == nil)
    }

    @Test
    func authenticationWithoutAnUnlockIsCancelled() {
        let password = MockKeystorePassword(memoryPassword: "stored", availableAuthentication: .biometrics)
        let keystorePassword = GemstoneKeystorePassword(password: password, unlockContext: UnlockContextStore())

        #expect(throws: GemServiceError.Cancelled) { try keystorePassword.getPassword(createIfMissing: false) }
        #expect(password.getPasswordCallsCount == 0)
    }

    @Test
    func authenticationReadsSilentlyWithTheUnlockContext() throws {
        let password = MockKeystorePassword(memoryPassword: "stored", availableAuthentication: .biometrics)
        let unlockContext = UnlockContextStore()
        let context = LAContext()
        unlockContext.set(context)
        let keystorePassword = GemstoneKeystorePassword(password: password, unlockContext: unlockContext)

        #expect(try keystorePassword.getPassword(createIfMissing: false) == "stored")
        #expect(password.passwordContext === context)
        #expect(context.interactionNotAllowed)
    }

    @Test
    func aClearedUnlockIsCancelled() {
        let password = MockKeystorePassword(memoryPassword: "stored", availableAuthentication: .biometrics)
        let unlockContext = UnlockContextStore()
        unlockContext.set(LAContext())
        unlockContext.clear()
        let keystorePassword = GemstoneKeystorePassword(password: password, unlockContext: unlockContext)

        #expect(throws: GemServiceError.Cancelled) { try keystorePassword.getPassword(createIfMissing: false) }
    }
}
