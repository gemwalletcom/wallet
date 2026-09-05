// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemAppUpdateServiceProtocol
import protocol Gemstone.GemTransactionStateServiceProtocol
import GemstonePrimitives
import protocol Gemstone.GemAppStartServiceProtocol
import AppService
import GemstoneServices
import Components
import protocol Gemstone.GemWalletSessionServiceProtocol
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
    private let appLifecycleService: AppLifecycleService
    private let navigationHandler: NavigationHandler
    private let appUpdateService: any GemAppUpdateServiceProtocol
    private let rateService: RateService
    private let toastPresenter: ToastPresenter
    private let deviceService: any GemDeviceServiceProtocol

    let observablePreferences: ObservablePreferences
    private let viewModelFactory: ViewModelFactory
    private let walletSessionService: any GemWalletSessionServiceProtocol
    let walletConnectorPresenter: WalletConnectorPresenter
    let lockManager: any LockWindowManageable

    var currentWallet: Wallet? {
        walletSessionService.currentWalletId.flatMap { try? viewModelFactory.storeManager.walletStore.getWallet(id: $0) }
    }
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
        appLifecycleService: AppLifecycleService,
        navigationHandler: NavigationHandler,
        lockWindowManager: any LockWindowManageable,
        viewModelFactory: ViewModelFactory,
        walletSessionService: any GemWalletSessionServiceProtocol,
        appUpdateService: any GemAppUpdateServiceProtocol,
        rateService: RateService,
        toastPresenter: ToastPresenter,
        deviceService: any GemDeviceServiceProtocol,
    ) {
        self.observablePreferences = observablePreferences
        self.walletConnectorPresenter = walletConnectorPresenter
        self.onstartService = onstartService
        self.appStartService = appStartService
        self.pushNotificationEnablerService = pushNotificationEnablerService
        self.appLifecycleService = appLifecycleService
        self.navigationHandler = navigationHandler
        lockManager = lockWindowManager
        self.viewModelFactory = viewModelFactory
        self.walletSessionService = walletSessionService
        self.appUpdateService = appUpdateService
        self.rateService = rateService
        self.toastPresenter = toastPresenter
        self.deviceService = deviceService
    }
}

// MARK: - Business Logic

extension RootSceneViewModel {
    func setup() {
        rateService.perform()
        Task { await checkForUpdate() }
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

    func createWalletModel() -> CreateWalletModel {
        viewModelFactory.createWalletScene(onComplete: { [weak self] in self?.dismissCreateWallet() })
    }

    func importWalletModel() -> ImportWalletViewModel {
        viewModelFactory.importWalletScene(onComplete: { [weak self] in self?.dismissImportWallet() })
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
            for failure in await appStartService.setupWallet(wallet: wallet.map()) {
                debugLog("wallet start \(failure.step) failed: \(failure.message)")
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
                    try await deviceService.synchronizeIfNeeded()
                }
            } catch {
                debugLog("requestPushPermissions error: \(error)")
            }
        }
    }
}
