// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemAppUpdateServiceProtocol
import protocol Gemstone.GemAvatarServiceProtocol
import protocol Gemstone.GemNameServiceProtocol
import GemstonePrimitives
import protocol Gemstone.GemAppStartServiceProtocol
import AppService
import GemstoneServices
import Components
import Foundation
import protocol Gemstone.GemDeviceServiceProtocol
import Localization
import LockManager
import Onboarding
import Preferences
import Primitives
import PrimitivesComponents
import SwiftUI
import WalletConnector

@Observable
@MainActor
final class RootSceneViewModel {
    private let onstartService: OnstartService
    private let appStartService: any GemAppStartServiceProtocol
    private let pushNotificationEnablerService: PushNotificationEnablerService
    private let transactionStateScheduler: TransactionStateScheduler
    private let appLifecycleService: AppLifecycleService
    private let navigationHandler: NavigationHandler
    private let appUpdateService: any GemAppUpdateServiceProtocol
    private let rateService: RateService
    private let toastPresenter: ToastPresenter
    private let deviceService: any GemDeviceServiceProtocol

    let observablePreferences: ObservablePreferences
    let walletService: WalletService
    let walletSessionService: any WalletSessionManageable
    let nameService: any GemNameServiceProtocol
    let avatarService: any GemAvatarServiceProtocol
    let walletConnectorPresenter: WalletConnectorPresenter
    let lockManager: any LockWindowManageable

    var currentWallet: Wallet? { walletSessionService.currentWallet }
    var currentWalletId: WalletId? { walletSessionService.currentWalletId }
    var colorScheme: ColorScheme? { observablePreferences.appearance.colorScheme }
    var updateVersionAlertMessage: AlertMessage?

    var isPresentingToastMessage: ToastMessage? {
        get { toastPresenter.toastMessage }
        set { toastPresenter.toastMessage = newValue }
    }

    var isPresentingConnectorError: String? {
        get { walletConnectorPresenter.isPresentingError }
        set { walletConnectorPresenter.isPresentingError = newValue }
    }

    var isPresentingConnectorSheet: WalletConnectorSheetType? {
        get { walletConnectorPresenter.isPresentingSheet }
        set { walletConnectorPresenter.isPresentingSheet = newValue }
    }

    var isPresentingConnectorBar: Bool {
        get { walletConnectorPresenter.isPresentingConnectionBar }
        set { walletConnectorPresenter.isPresentingConnectionBar = newValue }
    }

    var isPresentingCreateWalletSheet = false
    var isPresentingImportWalletSheet = false

    var toastOffset: CGFloat {
        UIDevice.current.userInterfaceIdiom == .phone ? .space32 + .space16 : .zero
    }

    init(
        observablePreferences: ObservablePreferences,
        walletConnectorPresenter: WalletConnectorPresenter,
        onstartService: OnstartService,
        appStartService: any GemAppStartServiceProtocol,
        pushNotificationEnablerService: PushNotificationEnablerService,
        transactionStateScheduler: TransactionStateScheduler,
        appLifecycleService: AppLifecycleService,
        navigationHandler: NavigationHandler,
        lockWindowManager: any LockWindowManageable,
        walletService: WalletService,
        walletSessionService: any WalletSessionManageable,
        nameService: any GemNameServiceProtocol,
        appUpdateService: any GemAppUpdateServiceProtocol,
        rateService: RateService,
        toastPresenter: ToastPresenter,
        avatarService: any GemAvatarServiceProtocol,
        deviceService: any GemDeviceServiceProtocol,
    ) {
        self.observablePreferences = observablePreferences
        self.walletConnectorPresenter = walletConnectorPresenter
        self.onstartService = onstartService
        self.appStartService = appStartService
        self.pushNotificationEnablerService = pushNotificationEnablerService
        self.transactionStateScheduler = transactionStateScheduler
        self.appLifecycleService = appLifecycleService
        self.navigationHandler = navigationHandler
        lockManager = lockWindowManager
        self.walletService = walletService
        self.walletSessionService = walletSessionService
        self.nameService = nameService
        self.appUpdateService = appUpdateService
        self.rateService = rateService
        self.toastPresenter = toastPresenter
        self.avatarService = avatarService
        self.deviceService = deviceService
    }
}

// MARK: - Business Logic

extension RootSceneViewModel {
    func setup() {
        rateService.perform()
        Task { await checkForUpdate() }
        Task { _ = try await deviceService.synchronize() }
        transactionStateScheduler.setup()
        Task { await appLifecycleService.setup() }
        Task { await setupWallets() }
    }

    func onScenePhaseChanged(_: ScenePhase, _ newPhase: ScenePhase) {
        Task {
            await appLifecycleService.handleScenePhase(newPhase)
        }
    }

    func onPerpetualEnabledChanged(_: Bool, _: Bool) {
        Task {
            await appLifecycleService.updatePerpetualConnection()
        }
    }
}

// MARK: - Effects

extension RootSceneViewModel {
    func onChangeWalletId() {
        guard let currentWallet else {
            Task { await appLifecycleService.updateWalletConnections() }
            return
        }
        navigationHandler.resetNavigation()
        setup(wallet: currentWallet)
    }

    func handleOpenUrl(_ url: URL) async {
        await navigationHandler.handle(url: url)
    }

    func dismissCreateWallet() {
        isPresentingCreateWalletSheet = false
        requestPushPermissions()
    }

    func dismissImportWallet() {
        isPresentingImportWalletSheet = false
        requestPushPermissions()
    }
}

// MARK: - Private

extension RootSceneViewModel {
    private func setup(wallet: Wallet) {
        Task {
            do {
                for failure in try await appStartService.setupWallet(wallet: wallet.json()) {
                    debugLog("wallet start \(failure.step) failed: \(failure.message)")
                }
            } catch {
                debugLog("RootSceneViewModel setupWallet error: \(error)")
            }
            await appLifecycleService.updateWalletConnections()
        }
    }

    private func setupWallets() async {
        await lockManager.lockModel.waitUntilUnlocked()
        await onstartService.setupWallets()
    }

    private func checkForUpdate() async {
        do {
            guard let release = try await appUpdateService.checkForUpdate() else { return }
            updateVersionAlertMessage = makeUpdateAlert(for: release)
        } catch {
            debugLog("checkForUpdate error: \(error)")
        }
    }

    private func makeUpdateAlert(for release: Release) -> AlertMessage {
        let skipAction = AlertAction(
            title: Localized.Common.skip,
            role: .cancel,
            action: { [appUpdateService] in
                do {
                    try appUpdateService.skip(version: release.version)
                } catch {
                    debugLog("skipRelease error: \(error)")
                }
            },
        )
        let updateAction = AlertAction(
            title: Localized.UpdateApp.action,
            isDefaultAction: true,
            action: {
                Task { @MainActor in
                    UIApplication.shared.open(AppUrl.page(.appStore))
                }
            },
        )
        let actions = release.upgradeRequired ? [updateAction] : [skipAction, updateAction]

        return AlertMessage(
            title: Localized.UpdateApp.title,
            message: Localized.UpdateApp.description(release.version),
            actions: actions,
        )
    }

    private func requestPushPermissions() {
        Task {
            do {
                if try await pushNotificationEnablerService.requestPermissionsIfNotDetermined() {
                    _ = try await deviceService.synchronize()
                }
            } catch {
                debugLog("requestPushPermissions error: \(error)")
            }
        }
    }
}
