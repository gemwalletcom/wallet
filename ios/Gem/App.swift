// Copyright (c). Gem Wallet. All rights reserved.

import AppService
import GemstoneServices
import LockManager
import Preferences
import Primitives
import Store
import Style
import SwiftUI

@main
struct GemApp: App {
    @UIApplicationDelegateAdaptor(AppDelegate.self) var appDelegate

    private let resolver: AppResolver = .main

    init() {
        UNUserNotificationCenter.current().delegate = appDelegate
    }

    var body: some Scene {
        WindowGroup {
            RootScene(
                model: RootSceneViewModel(
                    observablePreferences: resolver.services.observablePreferences,
                    walletConnectorPresenter: resolver.services.walletConnectorManager.presenter,
                    onstartService: resolver.services.onstartService,
                    appStartService: resolver.services.appStartService,
                    pushNotificationEnablerService: resolver.services.pushNotificationEnablerService,
                    transactionStateTracker: resolver.services.transactionStateTracker,
                    appLifecycleService: resolver.services.appLifecycleService,
                    navigationHandler: resolver.services.navigationHandler,
                    lockWindowManager: LockWindowManager(lockModel: LockSceneViewModel()),
                    walletService: resolver.services.walletService,
                    walletSessionService: resolver.services.walletSessionService,
                    nameService: resolver.services.nameService,
                    appUpdateService: resolver.services.appUpdateService,
                    rateService: resolver.services.rateService,
                    toastPresenter: resolver.services.toastPresenter,
                    avatarService: resolver.services.avatarService,
                    deviceService: resolver.services.deviceService,
                ),
            )
            .databaseQueue(resolver.storages.db.dbQueue)
            .navigationBarTitleDisplayMode(.inline)
            .tint(Colors.black)
        }
    }
}

class AppDelegate: NSObject, UIApplicationDelegate, UIWindowSceneDelegate {
    func application(_: UIApplication, didFinishLaunchingWithOptions _: [UIApplication.LaunchOptionsKey: Any]? = nil) -> Bool {
        AppResolver.main.services.onstartService.configure()
        Task {
            for failure in await AppResolver.main.services.appStartService.run() {
                debugLog("app start \(failure.step) failed: \(failure.message)")
            }
        }
        return true
    }

    func application(_: UIApplication, didRegisterForRemoteNotificationsWithDeviceToken deviceToken: Data) {
        let token = deviceToken.map { String(format: "%02.2hhx", $0) }.joined()

        Task {
            let _ = try SecurePreferences.standard.set(value: token, key: .deviceToken)
            _ = try await AppResolver.main.services.deviceService.synchronize()
        }
    }

    func application(_: UIApplication, didFailToRegisterForRemoteNotificationsWithError error: any Error) {
        debugLog("didFailToRegisterForRemoteNotificationsWithError error: \(error)")
    }

    func application(_: UIApplication, didReceiveRemoteNotification userInfo: [AnyHashable: Any]) {
        Task { await AppResolver.main.services.navigationHandler.handlePush(userInfo) }
    }

    func application(_: UIApplication, open url: URL, options _: [UIApplication.OpenURLOptionsKey: Any] = [:]) -> Bool {
        debugLog("url \(url)")
        return true
    }

    func scene(_: UIScene, openURLContexts _: Set<UIOpenURLContext>) {}

    func scene(_: UIScene, willConnectTo _: UISceneSession, options _: UIScene.ConnectionOptions) {}

    func application(_: UIApplication, shouldAllowExtensionPointIdentifier extensionPointIdentifier: UIApplication.ExtensionPointIdentifier) -> Bool {
        switch extensionPointIdentifier {
        case .keyboard: false
        default: true
        }
    }
}

extension AppDelegate: @preconcurrency UNUserNotificationCenterDelegate {
    func userNotificationCenter(_: UNUserNotificationCenter, willPresent _: UNNotification, withCompletionHandler completionHandler: @escaping (UNNotificationPresentationOptions) -> Void) {
        completionHandler([.badge, .banner, .list, .sound])
    }

    func userNotificationCenter(
        _: UNUserNotificationCenter,
        didReceive response: UNNotificationResponse,
        withCompletionHandler completionHandler: @escaping () -> Void,
    ) {
        Task { await AppResolver.main.services.navigationHandler.handlePush(response.notification.request.content.userInfo) }
        completionHandler()
    }
}
