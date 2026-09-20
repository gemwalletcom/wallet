// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemAppStartServiceProtocol
import protocol Gemstone.GemPreferencesServiceProtocol
import protocol Gemstone.GemWalletSessionServiceProtocol
import GemstonePrimitives
import GemstoneServices
import Primitives

/// OnstartService runs services before the app starts.
/// See OnstartAsyncService for any background tasks to run after start
public struct OnstartService: Sendable {
    private let appStartService: any GemAppStartServiceProtocol
    private let preferencesService: any GemPreferencesServiceProtocol
    private let keystore: any Keystore
    private let session: any GemWalletSessionServiceProtocol

    public init(
        appStartService: any GemAppStartServiceProtocol,
        preferencesService: any GemPreferencesServiceProtocol,
        keystore: any Keystore,
        session: any GemWalletSessionServiceProtocol,
    ) {
        self.appStartService = appStartService
        self.preferencesService = preferencesService
        self.keystore = keystore
        self.session = session
    }

    @MainActor
    public func configure() {
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

    @MainActor
    public func isDeviceCompromised() async -> Bool {
        #if DEBUG || targetEnvironment(simulator)
            return false
        #else
            if JailbreakChecks.hasJailbreakURLScheme() {
                return true
            }
            return await Task.detached {
                JailbreakChecks.hasSuspiciousPaths() || JailbreakChecks.canCreateFileOutsideSandbox() ||
                    JailbreakChecks.hasInjectedLibraries() || JailbreakChecks.hasOpenFridaPort()
            }.value
        #endif
    }

    public func setupWallets() async {
        do {
            let failures = try await keystore.migrateV3Keystores(for: session.getWallets())
            for failure in failures {
                debugLog("v3 keystore migration failed for \(failure.walletId.id): \(failure.error)")
            }
        } catch {
            debugLog("v3 keystore migration could not enumerate wallets: \(error)")
        }
        for failure in await appStartService.setupWallets() {
            debugLog("wallet setup \(failure.step) failed: \(failure.message)")
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

    private func configureScreenshots() {
        guard ProcessInfo.processInfo.environment["SCREENSHOTS_PATH"] != nil else { return }
        let currency = Locale.current.currency.flatMap { Currency(rawValue: $0.identifier) } ?? .usd
        do {
            try preferencesService.setCurrencyValue(currency)
        } catch {
            debugLog("screenshots currency error: \(error)")
        }
    }
}
