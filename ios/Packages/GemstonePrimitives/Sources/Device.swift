// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone

public func generateDeviceKeyPair() -> (privateKey: Data, publicKey: Data) {
    let keyPair = Gemstone.generateDeviceKeyPair()
    return (keyPair.privateKey, keyPair.publicKey)
}
