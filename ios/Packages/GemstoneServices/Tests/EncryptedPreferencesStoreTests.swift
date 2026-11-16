// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import GemstoneServices
import Keychain
import Testing

struct EncryptedPreferencesStoreTests {
    private let keychain = RecordingKeychain()
    private let directory = FileManager.default.temporaryDirectory.appending(path: UUID().uuidString, directoryHint: .isDirectory)

    @Test
    func encryptedValueLivesOnlyInTheAppContainer() throws {
        try store().set(key: "agent_key", value: "secret")

        #expect(try store().get(key: "agent_key") == "secret")
        #expect(try keychain.get("gatewayagent_key") == nil)
        #expect(keychain.storage.accessibility(for: "secure_store_key") == .whenUnlockedThisDeviceOnly)
        #expect(try directory.resourceValues(forKeys: [.isExcludedFromBackupKey]).isExcludedFromBackup == true)
        let files = try FileManager.default.contentsOfDirectory(at: directory, includingPropertiesForKeys: nil)
        #expect(try files.allSatisfy { try Data(contentsOf: $0).range(of: Data("secret".utf8)) == nil })

        try FileManager.default.removeItem(at: directory)

        #expect(try store().get(key: "agent_key") == nil)
    }

    @Test
    func keychainValueMovesIntoTheAppContainerOnFirstRead() throws {
        try keychain.set("secret", key: "gatewayagent_key")

        #expect(try store().get(key: "agent_key") == "secret")
        #expect(try keychain.get("gatewayagent_key") == nil)
        #expect(try store().get(key: "agent_key") == "secret")
    }

    @Test
    func removeDeletesEncryptedAndKeychainValues() throws {
        try store().set(key: "agent_address", value: "address")
        try keychain.set("secret", key: "gatewayagent_key")

        try store().remove(key: "agent_address")
        try store().remove(key: "agent_key")

        #expect(try store().get(key: "agent_address") == nil)
        #expect(try store().get(key: "agent_key") == nil)
    }

    @Test
    func valueFailsToDecryptUnderAnotherKey() throws {
        try store().set(key: "agent_key", value: "secret")
        try keychain.set(Data(repeating: 0, count: 32), key: "secure_store_key")

        #expect(throws: (any Error).self) { try store().get(key: "agent_key") }
    }

    private func store() -> GemstoneEncryptedPreferencesStore {
        GemstoneEncryptedPreferencesStore(namespace: "gateway", keychain: keychain, directory: directory)
    }
}
