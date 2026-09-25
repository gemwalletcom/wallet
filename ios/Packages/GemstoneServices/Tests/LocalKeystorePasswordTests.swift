// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import GemstoneServices
import LocalAuthentication
import Primitives
import Testing

struct LocalKeystorePasswordTests {
    @Test
    func emptyPasswordIsNeverStored() throws {
        let storage = KeychainStorage()
        let keystorePassword = LocalKeystorePassword(keychain: RecordingKeychain(storage: storage))

        #expect(throws: KeystoreError.self) { try keystorePassword.setPassword("", authentication: .none) }

        #expect(storage.value(for: "password") == nil)
        #expect(storage.value(for: "password_authentication") == nil)
    }

    @Test
    func authenticationChangeWithoutPasswordKeepsThePolicy() throws {
        let storage = KeychainStorage()
        let keystorePassword = LocalKeystorePassword(keychain: RecordingKeychain(storage: storage))

        try keystorePassword.enableAuthentication(false, context: LAContext())

        #expect(storage.value(for: "password") == nil)
        #expect(try keystorePassword.getAuthentication() == .none)
        #expect(storage.value(for: "password_authentication") == Data("none".utf8))
    }

    @Test
    func passwordIsStoredUnderTheRequestedAuthentication() throws {
        let storage = KeychainStorage()
        let keystorePassword = LocalKeystorePassword(keychain: RecordingKeychain(storage: storage))

        try keystorePassword.setPassword("secret", authentication: .passcode)

        #expect(try keystorePassword.getPassword() == "secret")
        #expect(try keystorePassword.getAuthentication() == .passcode)
    }

    @Test
    func creationKeepsThePasswordAnotherWriterStoredFirst() throws {
        let storage = KeychainStorage()
        let keystorePassword = LocalKeystorePassword(keychain: RecordingKeychain(storage: storage))
        try keystorePassword.setPassword("first", authentication: .none)

        #expect(try keystorePassword.createPassword("second", authentication: .none) == "first")
        #expect(try keystorePassword.getPassword() == "first")
    }

    @Test
    func creationReplacesAnEmptyLegacyPassword() throws {
        let storage = KeychainStorage()
        storage.set(Data(), key: "password", accessibility: .afterFirstUnlock)
        let keystorePassword = LocalKeystorePassword(keychain: RecordingKeychain(storage: storage))

        #expect(try keystorePassword.createPassword("new", authentication: .passcode) == "new")
        #expect(try keystorePassword.getPassword() == "new")
        #expect(try keystorePassword.getAuthentication() == .passcode)
    }

    @Test
    func creationStoresAnAbsentPassword() throws {
        let storage = KeychainStorage()
        let keystorePassword = LocalKeystorePassword(keychain: RecordingKeychain(storage: storage))

        #expect(try keystorePassword.createPassword("new", authentication: .none) == "new")
        #expect(try keystorePassword.getPassword() == "new")
        #expect(throws: KeystoreError.self) { try keystorePassword.createPassword("", authentication: .none) }
    }
}
