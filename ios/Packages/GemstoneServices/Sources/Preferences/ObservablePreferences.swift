// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemPreferencesObserver
import protocol Gemstone.GemPreferencesServiceProtocol
import GemstonePrimitives
import Observation
import Primitives

@Observable
public final class ObservablePreferences: Sendable {
    private let preferencesService: any GemPreferencesServiceProtocol

    public init(preferencesService: any GemPreferencesServiceProtocol) {
        self.preferencesService = preferencesService
        preferencesService.setObserver(observer: PreferencesObserver(preferences: self))
    }

    @ObservationIgnored
    public var currency: Primitives.Currency {
        get {
            access(keyPath: \.currency)
            return preferencesService.getCurrency().toPrimitives()
        }
        set {
            withMutation(keyPath: \.currency) {
                write { try preferencesService.setCurrency(currency: newValue.toGem()) }
            }
        }
    }

    @ObservationIgnored
    public var isHideBalanceEnabled: Bool {
        get {
            access(keyPath: \.isHideBalanceEnabled)
            return preferencesService.isHideBalanceEnabled()
        }
        set {
            withMutation(keyPath: \.isHideBalanceEnabled) {
                write { try preferencesService.setHideBalanceEnabled(enabled: newValue) }
            }
        }
    }

    @ObservationIgnored
    public var isDeveloperEnabled: Bool {
        get {
            access(keyPath: \.isDeveloperEnabled)
            return preferencesService.isDeveloperEnabled()
        }
        set {
            withMutation(keyPath: \.isDeveloperEnabled) {
                write { try preferencesService.setDeveloperEnabled(enabled: newValue) }
            }
        }
    }

    @ObservationIgnored
    public var isAcceptTermsCompleted: Bool {
        access(keyPath: \.isAcceptTermsCompleted)
        return preferencesService.isAcceptTermsCompleted()
    }

    @ObservationIgnored
    public var changes: Void {
        access(keyPath: \.changes)
    }

    public func reload() {
        withMutation(keyPath: \.changes) {}
        withMutation(keyPath: \.currency) {}
        withMutation(keyPath: \.isHideBalanceEnabled) {}
        withMutation(keyPath: \.isDeveloperEnabled) {}
        withMutation(keyPath: \.isAcceptTermsCompleted) {}
        withMutation(keyPath: \.isPerpetualEnabled) {}
        withMutation(keyPath: \.appearance) {}
    }

    public func acceptTerms() {
        withMutation(keyPath: \.isAcceptTermsCompleted) {
            write { try preferencesService.setAcceptTermsCompleted() }
        }
    }

    @ObservationIgnored
    public var isPerpetualEnabled: Bool {
        get {
            access(keyPath: \.isPerpetualEnabled)
            return preferencesService.isPerpetualEnabled()
        }
        set {
            withMutation(keyPath: \.isPerpetualEnabled) {
                write { try preferencesService.setPerpetualEnabled(enabled: newValue) }
            }
        }
    }

    @ObservationIgnored
    public var appearance: Appearance {
        get {
            access(keyPath: \.appearance)
            return preferencesService.getAppearance().toPrimitives()
        }
        set {
            withMutation(keyPath: \.appearance) {
                write { try preferencesService.setAppearance(appearance: newValue.toGem()) }
            }
        }
    }

    private func write(_ operation: () throws -> Void) {
        do {
            try operation()
        } catch {
            debugLog("preferences write error: \(error)")
        }
    }
}

private final class PreferencesObserver: GemPreferencesObserver, @unchecked Sendable {
    private weak var preferences: ObservablePreferences?

    init(preferences: ObservablePreferences) {
        self.preferences = preferences
    }

    func onPreferencesChanged() {
        Task { @MainActor [weak preferences] in
            preferences?.reload()
        }
    }
}
