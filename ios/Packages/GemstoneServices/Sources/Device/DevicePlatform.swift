// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemDeviceInfo
import protocol Gemstone.GemDevicePlatform
import GemstonePrimitives
import Preferences
import Primitives
import UIKit

public final class GemstoneDevicePlatform: GemDevicePlatform, @unchecked Sendable {
    private let preferences: Preferences
    private let securePreferences: SecurePreferences
    private let os: String
    private let model: String

    @MainActor
    public init(
        preferences: Preferences = .standard,
        securePreferences: SecurePreferences = SecurePreferences(),
    ) {
        self.preferences = preferences
        self.securePreferences = securePreferences
        os = UIDevice.current.osName
        model = UIDevice.current.modelName
    }

    public func deviceId() async throws -> String {
        try securePreferences.getDeviceId()
    }

    public func deviceInfo() async throws -> GemDeviceInfo {
        try GemDeviceInfo(
            platform: Platform.ios.json(),
            platformStore: PlatformStore.current.json(),
            os: os,
            model: model,
            version: Bundle.main.releaseVersionNumber,
            locale: Locale.current.deviceLocale().json(),
        )
    }

    public func pushToken() async throws -> String {
        try securePreferences.get(key: .deviceToken) ?? .empty
    }

    public func isPushEnabled() async throws -> Bool {
        preferences.isPushNotificationsEnabled
    }

    public func currency() async throws -> String {
        try Currency(id: preferences.currency).json()
    }
}
