// Copyright (c). Gem Wallet. All rights reserved.

@testable import Keychain
import LocalAuthentication
import Testing

struct KeychainTests {
    private let keychain: Keychain = KeychainDefault()

    @Test
    func basicStringOperations() throws {
        let key = "testKey"
        let value = "testValue"

        try keychain.set(value, key: key)
        #expect(try keychain.get(key) == value)

        try keychain.remove(key)
        #expect(try keychain.get(key) == nil)
    }

    @Test
    func dataOperations() throws {
        let key = "testDataKey"
        let value = "testDataValue".data(using: .utf8)!

        try keychain.set(value, key: key)
        #expect(try keychain.getData(key) == value)

        try keychain.remove(key)
        #expect(try keychain.getData(key) == nil)
    }

    @Test
    func accessibilityOptions() throws {
        let key = "testAccessibilityKey"
        let value = "testValue"

        let securedKeychain = keychain.accessibility(.whenUnlocked, authenticationPolicy: .userPresence)
        try securedKeychain.set(value, key: key)
        #expect(try securedKeychain.get(key) == value)

        try securedKeychain.remove(key)
        #expect(try securedKeychain.get(key) == nil)
    }

    @Test
    func testAuthenticationContext() throws {
        let key = "testAuthKey"
        let value = "testValue"
        let context = LAContext()

        let authKeychain = keychain.authenticationContext(context)
        try authKeychain.set(value, key: key)
        #expect(try authKeychain.get(key) == value)

        try authKeychain.remove(key)
        #expect(try authKeychain.get(key) == nil)
    }

    @Test
    func synchronizationOptions() throws {
        let key = "testSyncKey"
        let value = "testValue"

        try keychain.set(value, key: key, ignoringAttributeSynchronizable: true)
        #expect(try keychain.get(key, ignoringAttributeSynchronizable: true) == value)

        try keychain.remove(key)
        #expect(try keychain.get(key, ignoringAttributeSynchronizable: true) == nil)
    }

    @Test
    func multipleOperations() throws {
        let keys = ["key1", "key2", "key3"]
        let values = ["value1", "value2", "value3"]

        for (key, value) in zip(keys, values) {
            try keychain.set(value, key: key)
            #expect(try keychain.get(key) == value)
        }

        for key in keys {
            try keychain.remove(key)
            #expect(try keychain.get(key) == nil)
        }
    }

    @Test
    func setInvalidData() throws {
        let key = "invalidDataKey"
        let invalidData = Data([0xFF, 0xFF, 0xFF])

        #expect(throws: Status.conversionError) {
            try keychain.set(invalidData, key: key)
        }
    }

    @Test
    func getNonExistentKey() throws {
        let key = "nonExistentKey"
        #expect(try keychain.get(key) == nil)
    }

    @Test
    func removeNonExistentKey() throws {
        let key = "nonExistentKey"
        try keychain.remove(key)
    }
}
