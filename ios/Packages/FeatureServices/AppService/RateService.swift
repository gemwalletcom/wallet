// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemPreferencesServiceProtocol
import StoreKit

public struct RateService: Sendable {
    private let preferencesService: any GemPreferencesServiceProtocol

    public init(preferencesService: any GemPreferencesServiceProtocol) {
        self.preferencesService = preferencesService
    }

    public func perform() {
        #if targetEnvironment(simulator)
        #else
            do {
                guard try preferencesService.shouldRequestReview() else { return }
            } catch {
                debugLog("RateService: review preference read failed: \(error)")
                return
            }
            Task { @MainActor in
                if rate() {
                    do {
                        try preferencesService.setRateApplicationShown()
                    } catch {
                        debugLog("RateService: review preference write failed: \(error)")
                    }
                }
            }
        #endif
    }

    @MainActor
    @discardableResult
    private func rate() -> Bool {
        guard let scene = UIApplication.shared
            .connectedScenes
            .first(where: { $0.activationState == .foregroundActive }) as? UIWindowScene
        else { return false }
        AppStore.requestReview(in: scene)
        return true
    }
}
