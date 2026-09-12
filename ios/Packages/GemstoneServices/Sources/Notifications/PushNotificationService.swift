// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemPreferencesServiceProtocol
import Foundation
import UIKit

public struct PushNotificationEnablerService: Sendable {
    private let preferencesService: any GemPreferencesServiceProtocol

    public init(preferencesService: any GemPreferencesServiceProtocol) {
        self.preferencesService = preferencesService
    }

    private func requestPermissions() async throws -> Bool {
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
        switch preferencesService.notificationPrompt(isGranted: status.isGranted) {
        case .enable:
            try preferencesService.setPushNotificationsEnabled(enabled: true)
            await registerForRemoteNotifications()
            return true
        case .request:
            return try await requestPermissions()
        case .openSettings:
            try await openSetting()
            return false
        }
    }

    public func requestPermissionsIfNotDetermined() async throws -> Bool {
        switch preferencesService.notificationPrompt(isGranted: try await getNotificationSettingsStatus().isGranted) {
        case .request: try await requestPermissions()
        case .enable, .openSettings: false
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

private extension UNAuthorizationStatus {
    var isGranted: Bool {
        switch self {
        case .authorized, .ephemeral, .provisional: true
        case .denied, .notDetermined: false
        @unknown default: false
        }
    }
}
