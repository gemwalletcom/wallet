// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import GemstoneServices
import GemstoneServicesTestKit
import Primitives
import Testing

struct KeystoreBenchmarkTests {
    private static let iterations = 3

    @Test
    func benchmarkEncryptAndDecryptWithDefaultKdf() async throws {
        let keystore = LocalKeystore.mock(keystorePassword: MockKeystorePassword(memoryPassword: LocalKeystore.password))
        let clock = ContinuousClock()

        var encryptDurations: [Duration] = []
        var wallet: Primitives.Wallet?
        for index in 0 ... Self.iterations {
            if let wallet {
                _ = try keystore.gemKeystore.delete(keystoreId: keystore.gemKeystore.keystoreId(walletId: wallet.id.id))
            }
            let start = clock.now
            wallet = try keystore.importWallet(
                name: "Benchmark",
                type: .multicoinPhrase(words: LocalKeystore.words, chains: [Primitives.Chain.ethereum].map { $0.map() }),
            )
            if index > 0 {
                encryptDurations.append(clock.now - start)
            }
        }

        let imported = try #require(wallet)
        var decryptDurations: [Duration] = []
        for index in 0 ... Self.iterations {
            let start = clock.now
            let words = try await keystore.getMnemonic(wallet: imported)
            if index > 0 {
                decryptDurations.append(clock.now - start)
            }
            #expect(words == LocalKeystore.words)
        }
        _ = try keystore.gemKeystore.delete(keystoreId: keystore.gemKeystore.keystoreId(walletId: imported.id.id))

        print("keystore_v4 encrypt(importWallet) median: \(Self.median(encryptDurations))")
        print("keystore_v4 decrypt(getMnemonic) median: \(Self.median(decryptDurations))")
    }

    private static func median(_ durations: [Duration]) -> Duration {
        durations.sorted()[durations.count / 2]
    }
}
