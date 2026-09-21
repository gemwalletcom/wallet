// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.Deeplink
import protocol Gemstone.GemDeveloperServiceProtocol
import enum Gemstone.GemServiceError
import GemstonePrimitives
import class GemstoneServices.GemstoneDevicePlatform
import Localization
import Primitives
import PrimitivesComponents
import SwiftUI

@Observable
@MainActor
public final class DeveloperViewModel {
    private let walletId: WalletId
    private let service: any GemDeveloperServiceProtocol
    private let devicePlatform: GemstoneDevicePlatform

    public var isPresentingToastMessage: ToastMessage?
    public private(set) var deviceId: String = .empty
    public private(set) var deviceToken: String = .empty

    public init(
        walletId: WalletId,
        service: any GemDeveloperServiceProtocol,
        devicePlatform: GemstoneDevicePlatform,
    ) {
        self.walletId = walletId
        self.service = service
        self.devicePlatform = devicePlatform
    }

    var title: String {
        Localized.Settings.developer
    }

    func load() async {
        do {
            deviceId = try await service.deviceId()
        } catch let error as GemServiceError {
            deviceId = error.text().text
        } catch {
            debugLog("developer device id error: \(error)")
        }
        do {
            deviceToken = try await service.pushToken()
        } catch let error as GemServiceError {
            deviceToken = error.text().text
        } catch {
            debugLog("developer push token error: \(error)")
        }
    }

    func reset() {
        do {
            try clearDocuments()
            try service.clearPreferences()
            try devicePlatform.clearDeviceEntries()
            fatalError()
        } catch {
            debugLog("reset error \(error)")
        }
    }

    func clearCache() {
        toastResult {
            URLCache.shared.removeAllCachedResponses()
        }
    }

    func clearTransactions() {
        toastTaskResult { try await self.service.clearTransactions() }
    }

    func clearPendingTransactions() {
        toastTaskResult { try await self.service.clearPendingTransactions() }
    }

    func clearTransactionsTimestamp() {
        toastResult {
            try service.resetTransactionsTimestamp(walletId: walletId.id)
        }
    }

    func clearWalletPreferences() {
        toastResult {
            try service.deleteWalletPreferences(walletId: walletId.id)
        }
    }

    func clearAssets() {
        toastTaskResult { try await self.service.clearAssets() }
    }

    func clearDelegations() {
        toastTaskResult { try await self.service.clearDelegations() }
    }

    func clearValidators() {
        toastTaskResult { try await self.service.clearValidators() }
    }

    func clearBanners() {
        toastTaskResult { try await self.service.clearBanners() }
    }

    func activateAllCancelledBanners() {
        toastTaskResult { try await self.service.activateCancelledBanners() }
    }

    func clearPrices() {
        toastTaskResult { try await self.service.clearPrices() }
    }

    func clearPerpetuals() {
        toastTaskResult { try await self.service.clearPerpetualMarkets() }
    }

    func addTransactions() {
        toastTaskResult { try await self.service.addSampleTransactions(walletId: self.walletId.id) }
    }

    func deeplink(deeplink: Deeplink) {
        Task { @MainActor in
            await UIApplication.shared.open(service.deeplinkUrl(deeplink: deeplink).asURL!, options: [:])
        }
    }
}

// MARK: - Private

extension DeveloperViewModel {
    private func showSuccess() {
        isPresentingToastMessage = .success(Localized.Transaction.Status.confirmed)
    }

    private func toastResult(_ action: () throws -> Void) {
        do {
            try action()
            showSuccess()
        } catch {
            debugLog("Developer action error: \(error)")
        }
    }

    private func toastTaskResult(_ action: @escaping () async throws -> Void) {
        Task {
            do {
                try await action()
                showSuccess()
            } catch {
                debugLog("Developer action error: \(error)")
            }
        }
    }

    private func clearDocuments() throws {
        let documentsUrl = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask).first!
        let fileURLs = try FileManager.default.contentsOfDirectory(at: documentsUrl, includingPropertiesForKeys: nil, options: .skipsHiddenFiles)
        for fileURL in fileURLs {
            try FileManager.default.removeItem(at: fileURL)
        }
    }
}
