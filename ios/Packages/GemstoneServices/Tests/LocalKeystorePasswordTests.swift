// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import GemstoneServices
import Keychain
import LocalAuthentication
import Primitives
import Testing

struct LocalKeystorePasswordTests {
    @Test(arguments: [false, true])
    func missingPasswordIsNotCreatedByRead(legacyEmpty: Bool) throws {
        let storage = KeychainStorage()
        if legacyEmpty {
            storage.set(Data(), key: "password", accessibility: .afterFirstUnlock)
        }
        let keystorePassword = LocalKeystorePassword(keychain: RecordingKeychain(storage: storage))

        #expect(throws: KeystoreError.missingPassword) { try keystorePassword.getPassword() }

        #expect(storage.value(for: "password") == (legacyEmpty ? Data() : nil))
        #expect(storage.value(for: "password_authentication") == nil)
    }

    @Test
    func authenticationChangeWithoutPasswordKeepsThePolicy() throws {
        let storage = KeychainStorage()
        let keystorePassword = LocalKeystorePassword(keychain: RecordingKeychain(storage: storage))

        try keystorePassword.enableAuthentication(false, context: LAContext())

        #expect(storage.value(for: "password") == nil)
        #expect(try keystorePassword.getAuthentication() == .none)
    }

    @Test
    func creationUsesThePolicySelectedAfterAnotherCustodianRead() throws {
        let storage = KeychainStorage()
        let first = LocalKeystorePassword(keychain: RecordingKeychain(storage: storage))
        let second = LocalKeystorePassword(keychain: RecordingKeychain(storage: storage))
        #expect(try first.getAuthentication() == .none)
        storage.set(Data("passcode".utf8), key: "password_authentication", accessibility: .afterFirstUnlock)

        _ = try second.getPassword(createIfMissing: true)
        _ = try first.getPassword(createIfMissing: true)

        #expect(try first.getAuthentication() == .passcode)
        #expect(storage.authenticationPolicy(for: "password") == [.devicePasscode])
        #expect(storage.writeCount(for: "password") == 1)
        #expect(storage.writeCount(for: "password_authentication") == 1)
    }

    @Test
    func creationKeepsThePasswordAndPolicyAnotherWriterStoredFirst() throws {
        let storage = KeychainStorage()
        storage.set(Data("first".utf8), key: "password", accessibility: .whenUnlockedThisDeviceOnly, authenticationPolicy: [.devicePasscode])
        storage.set(Data("passcode".utf8), key: "password_authentication", accessibility: .afterFirstUnlock)
        let keystorePassword = LocalKeystorePassword(keychain: RecordingKeychain(storage: storage))

        #expect(try keystorePassword.getPassword(createIfMissing: true) == "first")
        #expect(try keystorePassword.getAuthentication() == .passcode)
        #expect(storage.authenticationPolicy(for: "password") == [.devicePasscode])
        #expect(storage.writeCount(for: "password") == 1)
        #expect(storage.writeCount(for: "password_authentication") == 1)
    }

    @Test
    func creationReplacesAnEmptyLegacyPasswordUnderTheStoredPolicy() throws {
        let storage = KeychainStorage()
        storage.set(Data(), key: "password", accessibility: .afterFirstUnlock)
        storage.set(Data("passcode".utf8), key: "password_authentication", accessibility: .afterFirstUnlock)
        let keystorePassword = LocalKeystorePassword(keychain: RecordingKeychain(storage: storage))

        let passwordLength = try keystorePassword.getPassword(createIfMissing: true).count

        #expect(passwordLength == 64)
        #expect(storage.authenticationPolicy(for: "password") == [.devicePasscode])
        #expect(try keystorePassword.getAuthentication() == .passcode)
    }

    @Test
    func concurrentCustodiansCreateOnePassword() async throws {
        let storage = KeychainStorage()
        let count = try await withThrowingTaskGroup(of: String.self) { group in
            for _ in 0 ..< 32 {
                group.addTask {
                    try LocalKeystorePassword(keychain: RecordingKeychain(storage: storage)).getPassword(createIfMissing: true)
                }
            }
            return try await group.reduce(into: Set<String>()) { $0.insert($1) }.count
        }

        #expect(count == 1)
        #expect(storage.writeCount(for: "password") == 1)
        #expect(storage.authenticationPolicy(for: "password") == [])
    }

    @Test
    func readFailureNeverCreatesOrChangesPassword() throws {
        let storage = KeychainStorage()
        storage.set(Data("first".utf8), key: "password", accessibility: .afterFirstUnlock)
        storage.setReadError(AnyError("unavailable"), key: "password")
        let keystorePassword = LocalKeystorePassword(keychain: RecordingKeychain(storage: storage))

        #expect(throws: AnyError("unavailable")) { try keystorePassword.getPassword(createIfMissing: true) }

        #expect(storage.writeCount(for: "password") == 1)
        #expect(storage.value(for: "password_authentication") == nil)
    }

    @Test
    func failedPolicyUpdateKeepsThePasswordProtection() throws {
        let storage = KeychainStorage()
        storage.set(Data("first".utf8), key: "password", accessibility: .whenUnlockedThisDeviceOnly, authenticationPolicy: [.devicePasscode])
        storage.set(Data("passcode".utf8), key: "password_authentication", accessibility: .afterFirstUnlock)
        storage.setWriteError(AnyError("unavailable"), key: "password_authentication")
        let keystorePassword = LocalKeystorePassword(keychain: RecordingKeychain(storage: storage))

        #expect(throws: AnyError("unavailable")) { try keystorePassword.enableAuthentication(false, context: LAContext()) }

        #expect(try keystorePassword.getAuthentication() == .passcode)
        #expect(storage.authenticationPolicy(for: "password") == [.devicePasscode])
    }

    @Test
    func unlockCreatesTheMissingLockKeyUnderTheStoredPolicy() throws {
        let storage = KeychainStorage()
        storage.set(Data("passcode".utf8), key: "password_authentication", accessibility: .afterFirstUnlock)
        let keystorePassword = LocalKeystorePassword(keychain: RecordingKeychain(storage: storage))

        try keystorePassword.unlock(context: LAContext())

        #expect(storage.value(for: "lock_key")?.count == 64)
        #expect(storage.accessibility(for: "lock_key") == .whenUnlockedThisDeviceOnly)
        #expect(storage.authenticationPolicy(for: "lock_key") == [.devicePasscode])
    }

    @Test
    func unlockKeepsTheStoredLockKey() throws {
        let storage = KeychainStorage()
        storage.set(Data("first".utf8), key: "lock_key", accessibility: .whenUnlockedThisDeviceOnly, authenticationPolicy: [.devicePasscode])
        let keystorePassword = LocalKeystorePassword(keychain: RecordingKeychain(storage: storage))

        try keystorePassword.unlock(context: LAContext())

        #expect(storage.value(for: "lock_key") == Data("first".utf8))
        #expect(storage.writeCount(for: "lock_key") == 1)
    }

    @Test
    func unreadableLockKeyFailsUnlockWithoutReplacingIt() throws {
        let storage = KeychainStorage()
        storage.set(Data("first".utf8), key: "lock_key", accessibility: .whenUnlockedThisDeviceOnly, authenticationPolicy: [.devicePasscode])
        storage.setReadError(AnyError("cancelled"), key: "lock_key")
        let keystorePassword = LocalKeystorePassword(keychain: RecordingKeychain(storage: storage))

        #expect(throws: AnyError("cancelled")) { try keystorePassword.unlock(context: LAContext()) }

        #expect(storage.value(for: "lock_key") == Data("first".utf8))
        #expect(storage.writeCount(for: "lock_key") == 1)
    }

    @Test
    func turningAuthenticationOffKeepsTheLockKeyWithoutProtection() throws {
        let storage = KeychainStorage()
        storage.set(Data("first".utf8), key: "lock_key", accessibility: .whenUnlockedThisDeviceOnly, authenticationPolicy: [.devicePasscode])
        storage.set(Data("passcode".utf8), key: "password_authentication", accessibility: .afterFirstUnlock)
        let keystorePassword = LocalKeystorePassword(keychain: RecordingKeychain(storage: storage))

        try keystorePassword.enableAuthentication(false, context: LAContext())

        #expect(storage.value(for: "lock_key") == Data("first".utf8))
        #expect(storage.authenticationPolicy(for: "lock_key") == [])
    }
}
