// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.Deeplink
import protocol Gemstone.GemDeveloperServiceProtocol
import Components
import Foundation
import GemstonePrimitives
import class GemstoneServices.SecurePreferences
import Localization
import Primitives
import SwiftUI

@Observable
@MainActor
public final class DeveloperViewModel {
    private let walletId: WalletId
    private let service: any GemDeveloperServiceProtocol

    public var isPresentingToastMessage: ToastMessage?
    public private(set) var deviceId: String = .empty
    public private(set) var deviceToken: String = .empty

    public init(
        walletId: WalletId,
        service: any GemDeveloperServiceProtocol,
    ) {
        self.walletId = walletId
        self.service = service
    }

    var title: String {
        Localized.Settings.developer
    }

    func load() async {
        deviceId = (try? await service.deviceId()) ?? .empty
        deviceToken = (try? await service.pushToken()) ?? .empty
    }

    func reset() {
        do {
            try clearDocuments()
            try service.clearPreferences()
            try SecurePreferences.standard.clear()
            fatalError()
        } catch {
            debugLog("reset error \(error)")
        }
    }

    func clearCache() {
        performAction {
            URLCache.shared.removeAllCachedResponses()
        }
    }

    func clearTransactions() {
        performTask { try await self.service.clearTransactions() }
    }

    func clearPendingTransactions() {
        performTask { try await self.service.clearPendingTransactions() }
    }

    func clearTransactionsTimestamp() {
        performAction {
            try service.resetTransactionsTimestamp(walletId: walletId.id)
        }
    }

    func clearWalletPreferences() {
        performAction {
            try service.deleteWalletPreferences(walletId: walletId.id)
        }
    }

    func clearAssets() {
        performTask { try await self.service.clearAssets() }
    }

    func clearDelegations() {
        performTask { try await self.service.clearDelegations() }
    }

    func clearValidators() {
        performTask { try await self.service.clearValidators() }
    }

    func clearBanners() {
        performTask { try await self.service.clearBanners() }
    }

    func activateAllCancelledBanners() {
        performTask { try await self.service.activateCancelledBanners() }
    }

    func clearPrices() {
        performTask { try await self.service.clearPrices() }
    }

    func clearPerpetuals() {
        performTask { try await self.service.clearPerpetualMarkets() }
    }

    func addTransactions() {
        performTask { try await self.service.addSampleTransactions(walletId: self.walletId.id) }
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

    private func performAction(_ action: () throws -> Void) {
        do {
            try action()
            showSuccess()
        } catch {
            debugLog("Developer action error: \(error)")
        }
    }

    private func performTask(_ action: @escaping () async throws -> Void) {
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
