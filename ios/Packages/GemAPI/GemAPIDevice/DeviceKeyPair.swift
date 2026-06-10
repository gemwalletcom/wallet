// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
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
        let keyPair = generateDeviceKeyPair()
        privateKey = keyPair.privateKey
        publicKey = keyPair.publicKey
    }
}
