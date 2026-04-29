// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import Preferences
import PreferencesTestKit
import Primitives
import Testing

struct SecurePreferencesTests {
    private let preferences: SecurePreferences = .mock()
    private let mockDeviceId: String = "Device ID #1"
    private let mockDeviceToken: String = "Device Token #1"

    @Test
    func defaultPreferences() {
        #expect(throws: Never.self) {
            let deviceId = try preferences.get(key: .deviceId)
            let deviceToken = try preferences.get(key: .deviceToken)

            #expect(deviceId == nil)
            #expect(deviceToken == nil)
        }
    }

    @Test
    func updatePreferences() {
        #expect(throws: Never.self) {
            try preferences.set(value: mockDeviceId, key: .deviceId)
            let deviceId = try preferences.get(key: .deviceId)
            #expect(deviceId == mockDeviceId)
        }

        #expect(throws: Never.self) {
            try preferences.set(value: mockDeviceToken, key: .deviceToken)
            let deviceToken = try preferences.get(key: .deviceToken)
            #expect(deviceToken == mockDeviceToken)
        }

        #expect(throws: Never.self) {
            let deviceId = try preferences.get(key: .deviceId)
            #expect(deviceId == mockDeviceId)
        }
    }

    @Test
    func testDelete() {
        #expect(throws: Never.self) {
            try preferences.set(value: mockDeviceId, key: .deviceId)
            let deviceId = try preferences.get(key: .deviceId)
            #expect(deviceId == mockDeviceId)
            try preferences.delete(key: .deviceId)
        }

        #expect(throws: Never.self) {
            let deviceId = try preferences.get(key: .deviceId)
            #expect(deviceId == nil)
        }
    }

    @Test
    func getDeviceIdCreatesAndPersistsCurrentDeviceId() {
        #expect(throws: Never.self) {
            let deviceId = try preferences.getDeviceId()
            let publicKey = try preferences.getData(key: .devicePublicKey)

            #expect(deviceId.count == 64)
            #expect(try preferences.get(key: .deviceId) == deviceId)
            #expect(publicKey?.hex == deviceId)
        }
    }

    @Test
    func getDeviceIdPrefersCurrentPublicKey() {
        #expect(throws: Never.self) {
            let privateKey = Data(repeating: 0x04, count: 32)
            let publicKey = Data([0x01, 0x02, 0x03])
            try preferences.set(value: mockDeviceId, key: .deviceId)
            try preferences.set(value: privateKey, key: .devicePrivateKey)
            try preferences.set(value: publicKey, key: .devicePublicKey)

            #expect(try preferences.getDeviceId() == publicKey.hex)
            #expect(try preferences.get(key: .deviceId) == publicKey.hex)
        }
    }

    @Test
    func testClear() {
        #expect(throws: Never.self) {
            try preferences.set(value: mockDeviceId, key: .deviceId)
            let deviceId = try preferences.get(key: .deviceId)
            #expect(deviceId == mockDeviceId)

            try preferences.set(value: mockDeviceToken, key: .deviceToken)
            let deviceToken = try preferences.get(key: .deviceToken)
            #expect(deviceToken == mockDeviceToken)

            try preferences.clear()
        }

        #expect(throws: Never.self) {
            let deviceId = try preferences.get(key: .deviceId)
            let deviceToken = try preferences.get(key: .deviceToken)

            #expect(deviceId == nil)
            #expect(deviceToken == nil)
        }
    }
}
