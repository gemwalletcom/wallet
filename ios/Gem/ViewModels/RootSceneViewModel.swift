// Copyright (c). Gem Wallet. All rights reserved.

import AppService
import AvatarService
import Components
import DeviceService
import EventPresenterService
import Foundation
import Localization
import LockManager
import Onboarding
import Preferences
import Primitives
import SwiftUI
import TransactionsService
import TransactionStateService
import WalletConnector
import WalletService
import WalletSessionService

@Observable
@MainActor
final class RootSceneViewModel {
    private let onstartService: OnstartService
    private let onstartWalletService: OnstartWalletService
    private let transactionStateScheduler: TransactionStateScheduler
    private let appLifecycleService: AppLifecycleService
    private let navigationHandler: NavigationHandler
    private let releaseAlertService: ReleaseAlertService
    private let rateService: RateService
    private let eventPresenterService: EventPresenterService
    private let deviceService: any DeviceServiceable

    let observablePreferences: ObservablePreferences
    let walletSetupService: WalletSetupService
    let walletService: WalletService
    let walletSessionService: any WalletSessionManageable
    let nameService: any NameServiceable
    let avatarService: AvatarService
    let walletConnectorPresenter: WalletConnectorPresenter
    let lockManager: any LockWindowManageable
    var currentWallet: Wallet? {
        walletSessionService.currentWallet
    }

    var updateVersionAlertMessage: AlertMessage?

    var isPresentingToastMessage: ToastMessage? {
        get { eventPresenterService.toastPresenter.toastMessage }
        set { eventPresenterService.toastPresenter.toastMessage = newValue }
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
        onstartWalletService: OnstartWalletService,
        transactionStateScheduler: TransactionStateScheduler,
        appLifecycleService: AppLifecycleService,
        navigationHandler: NavigationHandler,
        lockWindowManager: any LockWindowManageable,
        walletService: WalletService,
        walletSessionService: any WalletSessionManageable,
        walletSetupService: WalletSetupService,
        nameService: any NameServiceable,
        releaseAlertService: ReleaseAlertService,
        rateService: RateService,
        eventPresenterService: EventPresenterService,
        avatarService: AvatarService,
        deviceService: any DeviceServiceable,
    ) {
        self.observablePreferences = observablePreferences
        self.walletConnectorPresenter = walletConnectorPresenter
        self.onstartService = onstartService
        self.onstartWalletService = onstartWalletService
        self.transactionStateScheduler = transactionStateScheduler
        self.appLifecycleService = appLifecycleService
        self.navigationHandler = navigationHandler
        lockManager = lockWindowManager
        self.walletService = walletService
        self.walletSessionService = walletSessionService
        self.walletSetupService = walletSetupService
        self.nameService = nameService
        self.releaseAlertService = releaseAlertService
        self.rateService = rateService
        self.eventPresenterService = eventPresenterService
        self.avatarService = avatarService
        self.deviceService = deviceService
    }
}

// MARK: - Business Logic

extension RootSceneViewModel {
    func setup() {
        rateService.perform()
        Task { await checkForUpdate() }
        Task { try await deviceService.update() }
        transactionStateScheduler.setup()
        Task { await appLifecycleService.setup() }
        Task { await migrateV3KeystoresThenSetupChains() }
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
    func onChangeWallet(_ oldWallet: Wallet?, _ newWallet: Wallet?) {
        guard let newWallet else { return }
        if oldWallet?.id != newWallet.id {
            navigationHandler.resetNavigation()
        }
        setup(wallet: newWallet)
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
        navigationHandler.wallet = wallet
        onstartWalletService.setup(wallet: wallet)
        do {
            try walletSetupService.setup(wallet: wallet)
        } catch {
            debugLog("RootSceneViewModel setupWallet error: \(error)")
        }
        Task {
            await appLifecycleService.setupWallet(wallet)
        }
    }

    private func migrateV3KeystoresThenSetupChains() async {
        await lockManager.lockModel.waitUntilUnlocked()
        await onstartService.migrateV3KeystoresThenSetupChains()
    }

    private func checkForUpdate() async {
        guard let release = await releaseAlertService.checkForUpdate() else { return }
        updateVersionAlertMessage = makeUpdateAlert(for: release)
    }

    private func makeUpdateAlert(for release: Release) -> AlertMessage {
        let skipAction = AlertAction(
            title: Localized.Common.skip,
            role: .cancel,
            action: { [releaseAlertService] in
                releaseAlertService.skipRelease(release)
            },
        )
        let updateAction = AlertAction(
            title: Localized.UpdateApp.action,
            isDefaultAction: true,
            action: { [releaseAlertService] in
                Task { @MainActor in
                    releaseAlertService.openAppStore()
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
            await onstartWalletService.requestPushPermissions()
        }
    }
}
