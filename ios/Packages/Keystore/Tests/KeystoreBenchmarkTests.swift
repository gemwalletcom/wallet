// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import Keystore
import KeystoreTestKit
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
                try await keystore.deleteKey(for: wallet)
            }
            let start = clock.now
            wallet = try await keystore.importWallet(
                name: "Benchmark",
                type: .phrase(words: LocalKeystore.words, chains: [.ethereum]),
                isWalletsEmpty: false,
                source: .import,
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
        try await keystore.deleteKey(for: imported)

        print("keystore_v4 encrypt(importWallet) median: \(Self.median(encryptDurations))")
        print("keystore_v4 decrypt(getMnemonic) median: \(Self.median(decryptDurations))")
    }

    private static func median(_ durations: [Duration]) -> Duration {
        durations.sorted()[durations.count / 2]
    }
}
