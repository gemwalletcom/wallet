// Copyright (c). Gem Wallet. All rights reserved.

import GemstoneServices
import Foundation
import protocol Gemstone.GemPreferencesServiceProtocol
import Preferences
import Primitives
import Store
import UIKit
import WalletService

/// OnstartService runs services before the app starts.
/// See OnstartAsyncService for any background tasks to run after start
public struct OnstartService: Sendable {
    private let assetStore: AssetStore
    private let nodeStore: NodeStore
    private let preferences: Preferences
    private let preferencesService: any GemPreferencesServiceProtocol
    private let walletService: WalletService

    public init(
        assetStore: AssetStore,
        nodeStore: NodeStore,
        preferences: Preferences,
        preferencesService: any GemPreferencesServiceProtocol,
        walletService: WalletService,
    ) {
        self.assetStore = assetStore
        self.nodeStore = nodeStore
        self.preferences = preferences
        self.preferencesService = preferencesService
        self.walletService = walletService
    }

    @MainActor
    public func configure() {
        validateDeviceSecurity()
        configureURLCache()
        do {
            try excludeDirectoriesFromBackup()
            try migrateAssets()
            try migratePriceAlertsPreference()
            configureDefaultCurrency()
        } catch {
            debugLog("configure error: \(error)")
        }
        preferences.incrementLaunchesCount()

        #if DEBUG
            configureScreenshots()
        #endif
    }

    public func migrateV3KeystoresThenSetupChains() async {
        do {
            try await walletService.migrateV3Keystores()
        } catch {
            debugLog("v3 keystore migration could not enumerate wallets: \(error)")
        }
        do {
            try walletService.setup(chains: AssetConfiguration.allChains)
        } catch {
            debugLog("wallet chains setup error: \(error)")
        }
    }
}

// MARK: - Private

extension OnstartService {
    private func migrateAssets() throws {
        try ImportAssetsService(
            assetStore: assetStore,
            preferences: preferences,
        ).migrate()
    }

    private func migratePriceAlertsPreference() throws {
        guard let enabled = preferences.removeLegacyPriceAlertsEnabled() else { return }
        try preferencesService.setPriceAlertsEnabled(enabled: enabled)
    }

    private func configureDefaultCurrency() {
        if !preferences.hasCurrency, let currency = Locale.current.currency {
            preferences.currency = (Currency(rawValue: currency.identifier) ?? .usd).rawValue
        }
    }

    private func configureURLCache() {
        URLCache.shared.memoryCapacity = 256_000_000 // ~256 MB memory space
        URLCache.shared.diskCapacity = 1_000_000_000 // ~1GB disk cache space
    }

    private func excludeDirectoriesFromBackup() throws {
        let excludedBackupDirectories: [FileManager.Directory] = [.documents, .applicationSupport, .library(.preferences)]
        for directory in excludedBackupDirectories {
            try FileManager.default.addSkipBackupAttributeToItemAtURL(directory.url)

            #if DEBUG
                debugLog("Excluded backup directory: \(directory.directory)")
            #endif
        }
    }

    @MainActor
    private func validateDeviceSecurity() {
        let device = UIDevice.current
        if !device.isSimulator, device.isJailBroken || device.isFridaDetected {
            fatalError()
        }
    }

    private func configureScreenshots() {
        if ProcessInfo.processInfo.environment["SCREENSHOTS_PATH"] != nil {
            if let currency = Locale.current.currency, let currency = Currency(rawValue: currency.identifier) {
                Preferences.standard.currency = currency.rawValue
            } else {
                Preferences.standard.currency = Currency.usd.rawValue
            }
        }
    }
}
