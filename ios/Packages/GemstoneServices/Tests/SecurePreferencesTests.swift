// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import GemstoneServices
import GemstoneServicesTestKit
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
