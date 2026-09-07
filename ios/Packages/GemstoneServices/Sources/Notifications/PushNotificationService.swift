// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemPreferencesServiceProtocol
import Foundation
import UIKit

public struct PushNotificationEnablerService: PushNotificationEnabler {
    private let preferencesService: any GemPreferencesServiceProtocol

    public init(preferencesService: any GemPreferencesServiceProtocol) {
        self.preferencesService = preferencesService
    }

    public func requestPermissions() async throws -> Bool {
        if !preferencesService.isPushNotificationsEnabled() {
            let enabled = try await requestAuthorizationPermissions()
            try preferencesService.setPushNotificationsEnabled(enabled: enabled)
            return enabled
        }
        await registerForRemoteNotifications()
        return true
    }

    public func requestPermissionsOrOpenSettings() async throws -> Bool {
        let status = try await getNotificationSettingsStatus()
        switch status {
        case .authorized, .ephemeral, .provisional:
            try preferencesService.setPushNotificationsEnabled(enabled: true)
            await registerForRemoteNotifications()
            return true
        case .notDetermined:
            return try await requestPermissions()
        case .denied:
            try await openSetting()
            return false
        @unknown default:
            return false
        }
    }

    public func requestPermissionsIfNotDetermined() async throws -> Bool {
        switch try await getNotificationSettingsStatus() {
        case .notDetermined: try await requestPermissions()
        case .authorized, .ephemeral, .provisional, .denied: false
        @unknown default: false
        }
    }

    public func getNotificationSettingsStatus() async throws -> UNAuthorizationStatus {
        let center = UNUserNotificationCenter.current()
        return await center.notificationSettings().authorizationStatus
    }

    func openSetting() async throws {
        if let appSettings = URL(string: UIApplication.openSettingsURLString) {
            if await UIApplication.shared.canOpenURL(appSettings) {
                await UIApplication.shared.open(appSettings, completionHandler: .none)
            }
        }
    }

    private func requestAuthorizationPermissions() async throws -> Bool {
        let result = try await UNUserNotificationCenter.current().requestAuthorization(options: [.badge, .sound, .alert])
        await registerForRemoteNotifications()
        return result
    }

    @MainActor
    private func registerForRemoteNotifications() {
        UIApplication.shared.registerForRemoteNotifications()
    }
}
