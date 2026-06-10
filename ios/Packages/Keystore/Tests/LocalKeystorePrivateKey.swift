// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemKeystore
import Primitives
@testable import Keystore

extension LocalKeystore {
    func getPrivateKey(wallet: Primitives.Wallet, chain: Primitives.Chain) async throws -> Data {
        let password = try await getPassword()
        return try withV4Password(password) { passwordBytes in
            try gemKeystore.privateKey(keystoreId: wallet.keystoreId, chain: chain.rawValue, password: passwordBytes)
        }
    }
}
