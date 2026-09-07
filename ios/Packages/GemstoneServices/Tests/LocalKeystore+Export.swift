// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import GemstoneServices
import Primitives

extension LocalKeystore {
    func exportedPrivateKey(wallet: Primitives.Wallet, chain: Primitives.Chain) async throws -> String {
        let password = try await getPassword()
        return try withV4Password(keystore: gemKeystore, password) { password in
            try gemKeystore.exportPrivateKey(keystoreId: gemKeystore.keystoreId(walletId: wallet.id.id), chain: chain.rawValue, password: password)
        }
    }

    func exportedWords(wallet: Primitives.Wallet) async throws -> [String] {
        let password = try await getPassword()
        return try withV4Password(keystore: gemKeystore, password) { password in
            try gemKeystore.exportRecoveryPhrase(keystoreId: gemKeystore.keystoreId(walletId: wallet.id.id), password: password)
        }
    }
}
