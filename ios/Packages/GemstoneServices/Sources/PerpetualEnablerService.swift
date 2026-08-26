// Copyright (c). Gem Wallet. All rights reserved.

import Preferences
import Primitives

public struct PerpetualEnablerService: Sendable {
    private let observer: any PerpetualObservable
    private let service: any PerpetualServiceable
    private let preferences: Preferences

    public init(
        observer: any PerpetualObservable,
        service: any PerpetualServiceable,
        preferences: Preferences,
    ) {
        self.observer = observer
        self.service = service
        self.preferences = preferences
    }

    public func updateConnection(wallet: Wallet?) async {
        if let wallet, preferences.showPerpetuals(for: wallet) {
            await observer.setup(for: wallet)
        } else {
            await observer.disconnect()
        }
    }

    public func disconnect() async {
        await observer.disconnect()
    }

    public func updateEnablement(wallet: Wallet?) async {
        if preferences.isPerpetualEnabled {
            await updateMarkets()
            await updateConnection(wallet: wallet)
        } else {
            await disconnect()
            await clearMarkets()
        }
    }

    private func updateMarkets() async {
        do {
            try await service.updateMarkets()
        } catch {
            debugLog("PerpetualEnablerService updateMarkets error: \(error)")
        }
    }

    private func clearMarkets() async {
        do {
            try service.clearMarkets()
        } catch {
            debugLog("PerpetualEnablerService clearMarkets error: \(error)")
        }
    }
}
