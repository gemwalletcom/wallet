// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemAppUpdateServiceProtocol
import GemstonePrimitives
import Primitives
import UIKit

public struct ReleaseAlertService: Sendable {
    private let appUpdateService: any GemAppUpdateServiceProtocol

    public init(appUpdateService: any GemAppUpdateServiceProtocol) {
        self.appUpdateService = appUpdateService
    }

    public func newestRelease() async -> Release? {
        try? await appUpdateService.newest(store: PlatformStore.current.json(), currentVersion: Bundle.main.releaseVersionNumber)
            .map { try Release($0) }
    }

    public func checkForUpdate() async -> Release? {
        do {
            return try await appUpdateService.check(store: PlatformStore.current.json(), currentVersion: Bundle.main.releaseVersionNumber)
                .map { try Release($0) }
        } catch {
            debugLog("checkForUpdate error: \(error)")
            return nil
        }
    }

    public func skipRelease(_ release: Release) {
        do {
            try appUpdateService.skip(version: release.version)
        } catch {
            debugLog("skipRelease error: \(error)")
        }
    }

    @MainActor
    public func openAppStore() {
        UIApplication.shared.open(AppUrl.page(.appStore))
    }
}
