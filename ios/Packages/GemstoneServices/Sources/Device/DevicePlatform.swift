// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.Currency
import struct Gemstone.GemDeviceInfo
import protocol Gemstone.GemDeviceKeyServiceProtocol
import protocol Gemstone.GemDevicePlatform
import protocol Gemstone.GemPreferencesServiceProtocol
import GemstonePrimitives
import Keychain
import Primitives
import UIKit
import UserNotifications

public final class GemstoneDevicePlatform: GemDevicePlatform, @unchecked Sendable {
    private enum Keys {
        static let deviceId = "deviceId"
        static let deviceToken = "deviceToken"
        static let devicePrivateKey = "devicePrivateKey"
        static let devicePublicKey = "devicePublicKey"
    }

    private let preferencesService: any GemPreferencesServiceProtocol
    private let deviceKeyService: any GemDeviceKeyServiceProtocol
    private let keychain: any Keychain
    private let os: String
    private let model: String

    @MainActor
    public init(
        preferencesService: any GemPreferencesServiceProtocol,
        deviceKeyService: any GemDeviceKeyServiceProtocol,
        keychain: any Keychain = KeychainDefault(),
    ) {
        self.preferencesService = preferencesService
        self.deviceKeyService = deviceKeyService
        self.keychain = keychain
        os = UIDevice.current.osName
        model = UIDevice.current.modelName
    }

    public func deviceId() async throws -> String {
        try deviceKeyService.deviceId()
    }

    public func deviceInfo() async throws -> GemDeviceInfo {
        GemDeviceInfo(
            platform: Platform.ios.toGem(),
            platformStore: PlatformStore.current.toGem(),
            os: os,
            model: model,
            version: Bundle.main.releaseVersionNumber,
            localeIdentifier: Locale.current.identifier(.bcp47),
        )
    }

    public func pushToken() async throws -> String {
        try keychain.get(Keys.deviceToken) ?? .empty
    }

    public func setPushToken(_ token: String) throws {
        try keychain.accessibility(.whenUnlockedThisDeviceOnly, authenticationPolicy: []).set(token, key: Keys.deviceToken)
    }

    public func clearDeviceEntries() throws {
        for key in [Keys.deviceId, Keys.deviceToken, Keys.devicePrivateKey, Keys.devicePublicKey] {
            try keychain.remove(key)
        }
    }

    public func isPushEnabled() async throws -> Bool {
        guard preferencesService.isPushNotificationsEnabled() else {
            return false
        }
        return await UNUserNotificationCenter.current().notificationSettings().authorizationStatus.isAuthorized
    }

    public func getCurrency() async throws -> Gemstone.Currency {
        preferencesService.getCurrency()
    }
}
