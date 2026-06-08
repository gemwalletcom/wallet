// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
@testable import Keystore
import KeystoreTestKit
import Primitives
import PrimitivesTestKit
import Testing

struct MigrateV3KeystoreTests {
    private static let password = LocalKeystore.password
    private static let ethereumAddress = "0x5a8f70b44aFa00Cb70615D9c9CCb9A24933ED2D3"
    private static let expectedPrivateKey = "ae8794f84919b14ff9d1f0f7cf490a4c04e608de16864f53fe8b40af127b9da3"

    @Test
    func migrateV3WalletKeyedByWalletId() async throws {
        let legacy = Wallet.mock(
            id: .privateKey(chain: .ethereum, address: Self.ethereumAddress),
            type: .privateKey,
            source: .import,
        )
        try await assertMigratesAndIsIdempotent(legacy)
    }

    @Test
    func migrateV3WalletKeyedByLegacyExternalId() async throws {
        let legacy = Wallet.mock(
            id: .privateKey(chain: .ethereum, address: Self.ethereumAddress),
            externalId: "d6604f82-9e31-47b3-81db-bab91ab9d72d",
            type: .privateKey,
            source: .import,
        )
        let keystoreId = try await assertMigratesAndIsIdempotent(legacy)
        #expect(keystoreId != legacy.externalId)
    }

    @Test
    func deleteAfterMigrationRemovesEveryCopy() async throws {
        let directory = "migrate-test-\(UUID().uuidString)"
        let baseDir = try FileManager.default
            .url(for: .applicationSupportDirectory, in: .userDomainMask, appropriateFor: nil, create: true)
            .appending(path: directory, directoryHint: .isDirectory)
        defer { try? FileManager.default.removeItem(at: baseDir) }

        let keystore = LocalKeystore(
            directory: directory,
            keystorePassword: MockKeystorePassword(memoryPassword: Self.password),
        )
        let legacy = Wallet.mock(
            id: .privateKey(chain: .ethereum, address: Self.ethereumAddress),
            type: .privateKey,
            source: .import,
        )
        let fixtureURL = try #require(Bundle.module.url(forResource: "v3_ios_private_key", withExtension: "json"))
        let v3URL = baseDir.appending(path: legacy.legacyV3Id, directoryHint: .notDirectory)
        try FileManager.default.copyItem(at: fixtureURL, to: v3URL)

        let keystoreId = try #require(try await keystore.migrateV3Keystore(for: legacy))
        let v4URL = baseDir.appending(path: "v4/\(keystoreId).gemk")
        #expect(FileManager.default.fileExists(atPath: v4URL.path))
        #expect(FileManager.default.fileExists(atPath: v3URL.path))

        try await keystore.deleteKey(for: legacy)
        #expect(!FileManager.default.fileExists(atPath: v4URL.path))
        #expect(!FileManager.default.fileExists(atPath: v3URL.path))
    }

    @discardableResult
    private func assertMigratesAndIsIdempotent(_ legacy: Wallet) async throws -> String {
        let directory = "migrate-test-\(UUID().uuidString)"
        let baseDir = try FileManager.default
            .url(for: .applicationSupportDirectory, in: .userDomainMask, appropriateFor: nil, create: true)
            .appending(path: directory, directoryHint: .isDirectory)
        defer { try? FileManager.default.removeItem(at: baseDir) }

        let keystore = LocalKeystore(
            directory: directory,
            keystorePassword: MockKeystorePassword(memoryPassword: Self.password),
        )

        let fixtureURL = try #require(Bundle.module.url(forResource: "v3_ios_private_key", withExtension: "json"))
        let v3URL = baseDir.appending(path: legacy.legacyV3Id, directoryHint: .notDirectory)
        try FileManager.default.copyItem(at: fixtureURL, to: v3URL)

        let keystoreId = try #require(try await keystore.migrateV3Keystore(for: legacy))
        #expect(keystoreId == legacy.keystoreId)
        #expect(FileManager.default.fileExists(atPath: baseDir.appending(path: "v4/\(keystoreId).gemk").path))
        #expect(FileManager.default.fileExists(atPath: v3URL.path))

        let key = try await keystore.getPrivateKey(wallet: legacy, chain: .ethereum)
        #expect(key.hex == Self.expectedPrivateKey)

        #expect(try await keystore.migrateV3Keystore(for: legacy) == nil)
        #expect(FileManager.default.fileExists(atPath: v3URL.path))
        let keyAgain = try await keystore.getPrivateKey(wallet: legacy, chain: .ethereum)
        #expect(keyAgain.hex == Self.expectedPrivateKey)

        return keystoreId
    }
}
