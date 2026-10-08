// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.Currency
import struct Gemstone.GemDeviceInfo
import protocol Gemstone.GemDevicePlatform
import protocol Gemstone.GemNotificationPermissions
import protocol Gemstone.GemStreamConnection
import Primitives

final class GemDevicePlatformMock: GemDevicePlatform, @unchecked Sendable {
    func deviceId() async throws -> String {
        throw AnyError.notImplemented
    }

    func deviceInfo() async throws -> GemDeviceInfo {
        throw AnyError.notImplemented
    }

    func pushToken() async throws -> String {
        throw AnyError.notImplemented
    }

    func isPushEnabled() async throws -> Bool {
        false
    }

    func getCurrency() async throws -> Currency {
        .usd
    }
}

final class GemNotificationPermissionsMock: GemNotificationPermissions, @unchecked Sendable {
    func isAvailable() -> Bool {
        false
    }

    func requestPermissionsOrOpenSettings() async throws -> Bool {
        false
    }
}

final class GemStreamConnectionMock: GemStreamConnection, @unchecked Sendable {
    func latency() async throws -> TimeInterval? {
        nil
    }

    func isConnected() async -> Bool {
        false
    }

    func send(message _: String) async throws {}
}
