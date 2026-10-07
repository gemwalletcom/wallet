// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemKeystoreAuthentication
import LocalAuthentication
@testable import PrimitivesComponents
import Style
import Testing

struct KeystoreAuthenticationImageTests {
    @Test
    func eachSensorHasItsOwnGlyphAndName() {
        #expect(LABiometryType.faceID.systemImage == SystemImage.faceid)
        #expect(LABiometryType.touchID.systemImage == SystemImage.touchid)
        #expect(LABiometryType.opticID.systemImage == SystemImage.opticid)
        #expect(LABiometryType.none.systemImage == nil)

        #expect(LABiometryType.faceID.name == "Face ID")
        #expect(LABiometryType.touchID.name == "Touch ID")
        #expect(LABiometryType.opticID.name == "Optic ID")
        #expect(LABiometryType.none.name == nil)
    }

    @Test
    func onlyBiometricsDependsOnTheSensor() {
        #expect(GemKeystoreAuthentication.passcode.systemImage == SystemImage.lock)
        #expect(GemKeystoreAuthentication.passcode.biometrySystemImage == nil)
        #expect(GemKeystoreAuthentication.passcode.biometryName == nil)
        #expect(GemKeystoreAuthentication.none.systemImage == nil)
        #expect(GemKeystoreAuthentication.none.biometryName == nil)
    }
}
