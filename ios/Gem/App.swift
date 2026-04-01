// Copyright (c). Gem Wallet. All rights reserved.

import AppService
import AssetsService
import DeviceService
import GemAPI
import LockManager
import NodeService
import Preferences
import Primitives
import Store
import Style
import SwiftUI
import WalletService

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
                    observablePreferences: resolver.storages.observablePreferences,
                    walletConnectorPresenter: resolver.services.walletConnectorManager.presenter,
                    onstartWalletService: resolver.services.onstartWalletService,
                    transactionStateService: resolver.services.transactionStateService,
                    connectionsService: resolver.services.connectionsService,
                    appLifecycleService: resolver.services.appLifecycleService,
                    navigationHandler: resolver.services.navigationHandler,
                    lockWindowManager: LockWindowManager(lockModel: LockSceneViewModel()),
                    walletService: resolver.services.walletService,
                    walletSetupService: resolver.services.walletSetupService,
                    nameService: resolver.services.nameService,
                    releaseAlertService: resolver.services.releaseAlertService,
                    rateService: resolver.services.rateService,
                    eventPresenterService: resolver.services.eventPresenterService,
                    avatarService: resolver.services.avatarService,
                    deviceService: resolver.services.deviceService,
                ),
            )
            .inject(resolver: resolver)
            .navigationBarTitleDisplayMode(.inline)
            .tint(Colors.black)
        }
    }
}

class AppDelegate: NSObject, UIApplicationDelegate, UIWindowSceneDelegate {
    func application(_: UIApplication, didFinishLaunchingWithOptions _: [UIApplication.LaunchOptionsKey: Any]? = nil) -> Bool {
        AppResolver.main.services.onstartService.configure()
        Task {
            await AppResolver.main.services.onstartAsyncService.run()
        }
        return true
    }

    func application(_: UIApplication, didRegisterForRemoteNotificationsWithDeviceToken deviceToken: Data) {
        let token = deviceToken.map { String(format: "%02.2hhx", $0) }.joined()

        Task {
            let _ = try SecurePreferences.standard.set(value: token, key: .deviceToken)
            try await AppResolver.main.services.deviceService.update()
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

    func scene(_: UIScene, openURLContexts _: Set<UIOpenURLContext>) {
        // debugLog("URLContexts.first?.url \(URLContexts.first?.url)")
    }

    func scene(_: UIScene, willConnectTo _: UISceneSession, options _: UIScene.ConnectionOptions) {
        // debugLog("URLContexts.first?.url \(connectionOptions.urlContexts.first?.url)")
    }

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
