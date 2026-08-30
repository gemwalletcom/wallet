// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemDeviceKeyService
import struct Gemstone.GemDeviceInfo
import protocol Gemstone.GemDevicePlatform
import protocol Gemstone.GemPreferencesServiceProtocol
import GemstonePrimitives
import Preferences
import Primitives
import UIKit
import UserNotifications

public final class GemstoneDevicePlatform: GemDevicePlatform, @unchecked Sendable {
    private let preferencesService: any GemPreferencesServiceProtocol
    private let deviceKeyService: GemDeviceKeyService
    private let securePreferences: SecurePreferences
    private let os: String
    private let model: String

    @MainActor
    public init(
        preferencesService: any GemPreferencesServiceProtocol,
        deviceKeyService: GemDeviceKeyService,
        securePreferences: SecurePreferences = SecurePreferences(),
    ) {
        self.preferencesService = preferencesService
        self.deviceKeyService = deviceKeyService
        self.securePreferences = securePreferences
        os = UIDevice.current.osName
        model = UIDevice.current.modelName
    }

    public func deviceId() async throws -> String {
        try deviceKeyService.deviceId()
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
        guard preferencesService.isPushNotificationsEnabled() else {
            return false
        }
        return await UNUserNotificationCenter.current().notificationSettings().authorizationStatus.isAuthorized
    }

    public func currency() async throws -> String {
        preferencesService.getCurrency()
    }
}
