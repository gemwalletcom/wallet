// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemServiceError
import GemstoneServices
import GemstoneServicesTestKit
import LocalAuthentication
import Testing

struct LightningKeystorePasswordTests {
    @Test
    func readsThePasswordWithoutAuthentication() throws {
        let password = MockKeystorePassword(memoryPassword: "stored", availableAuthentication: .none)
        let lightning = LightningKeystorePassword(password: password, unlockContext: UnlockContextStore())

        #expect(try lightning.getPassword(createIfMissing: false) == "stored")
        #expect(password.passwordContext == nil)
    }

    @Test
    func authenticationWithoutAnUnlockIsCancelled() {
        let password = MockKeystorePassword(memoryPassword: "stored", availableAuthentication: .biometrics)
        let lightning = LightningKeystorePassword(password: password, unlockContext: UnlockContextStore())

        #expect(throws: GemServiceError.Cancelled) { try lightning.getPassword(createIfMissing: false) }
        #expect(password.getPasswordCallsCount == 0)
    }

    @Test
    func authenticationReadsSilentlyWithTheUnlockContext() throws {
        let password = MockKeystorePassword(memoryPassword: "stored", availableAuthentication: .biometrics)
        let unlockContext = UnlockContextStore()
        let context = LAContext()
        unlockContext.set(context)
        let lightning = LightningKeystorePassword(password: password, unlockContext: unlockContext)

        #expect(try lightning.getPassword(createIfMissing: false) == "stored")
        #expect(password.passwordContext === context)
        #expect(context.interactionNotAllowed)
    }

    @Test
    func aClearedUnlockIsCancelled() {
        let password = MockKeystorePassword(memoryPassword: "stored", availableAuthentication: .biometrics)
        let unlockContext = UnlockContextStore()
        unlockContext.set(LAContext())
        unlockContext.clear()
        let lightning = LightningKeystorePassword(password: password, unlockContext: unlockContext)

        #expect(throws: GemServiceError.Cancelled) { try lightning.getPassword(createIfMissing: false) }
    }
}
