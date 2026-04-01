// Copyright (c). Gem Wallet. All rights reserved.

import CryptoKit
import Foundation
import Primitives

public struct DeviceKeyPair: Sendable {
    public let privateKey: Data
    public let publicKey: Data

    public var privateKeyHex: String {
        privateKey.hex
    }

    public var publicKeyHex: String {
        publicKey.hex
    }

    public init() {
        let privateKey = Curve25519.Signing.PrivateKey()
        self.privateKey = privateKey.rawRepresentation
        publicKey = privateKey.publicKey.rawRepresentation
    }
}
