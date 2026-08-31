// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import GemstoneServices
import Foundation
import protocol Gemstone.GemAppStartServiceProtocol
import protocol Gemstone.GemPreferencesServiceProtocol
import Primitives
import UIKit

/// OnstartService runs services before the app starts.
/// See OnstartAsyncService for any background tasks to run after start
public struct OnstartService: Sendable {
    private let appStartService: any GemAppStartServiceProtocol
    private let preferencesService: any GemPreferencesServiceProtocol
    private let walletService: WalletService

    public init(
        appStartService: any GemAppStartServiceProtocol,
        preferencesService: any GemPreferencesServiceProtocol,
        walletService: WalletService,
    ) {
        self.appStartService = appStartService
        self.preferencesService = preferencesService
        self.walletService = walletService
    }

    @MainActor
    public func configure() {
        validateDeviceSecurity()
        configureURLCache()
        do {
            try excludeDirectoriesFromBackup()
            _ = try preferencesService.setupCurrency(localeCurrency: Locale.current.currency?.identifier)
            _ = try preferencesService.incrementLaunchesCount()
        } catch {
            debugLog("configure error: \(error)")
        }

        #if DEBUG
            configureScreenshots()
        #endif
    }

    public func setupWallets() async {
        do {
            try await walletService.migrateV3Keystores()
        } catch {
            debugLog("v3 keystore migration could not enumerate wallets: \(error)")
        }
        do {
            _ = try await appStartService.setupWallets()
        } catch {
            debugLog("wallet setup error: \(error)")
        }
    }
}

// MARK: - Private

extension OnstartService {
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
        guard ProcessInfo.processInfo.environment["SCREENSHOTS_PATH"] != nil else { return }
        let currency = Locale.current.currency.flatMap { Currency(rawValue: $0.identifier) } ?? .usd
        do {
            try preferencesService.setCurrency(currency: currency.json())
        } catch {
            debugLog("screenshots currency error: \(error)")
        }
    }
}
