// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Keystore
import KeystoreTestKit
import PrimitivesTestKit
@testable import Signer
import Testing

let TestPrivateKey = Data(hexString: "1E9D38B5274152A78DFF1A86FA464CEADC1F4238CA2C17060C3C507349424A34")!

struct SignerTests {
    @Test
    func signMessage() {
        let signer = Signer(wallet: .mock(), keystore: LocalKeystore.mock()).signer(for: .ethereum)

        #expect(type(of: signer) == ChainSigner.self)
    }
}
