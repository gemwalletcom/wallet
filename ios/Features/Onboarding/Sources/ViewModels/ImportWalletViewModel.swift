// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemNameServiceProtocol
import protocol Gemstone.GemWalletServiceProtocol
import GemstoneServices
import Primitives
import PrimitivesComponents
import SwiftUI

@Observable
@MainActor
public final class ImportWalletViewModel {
    private let service: any GemWalletServiceProtocol
    private let biometryService: any BiometryAuthenticatable
    let preferences: ObservablePreferences
    private let nameService: any GemNameServiceProtocol
    let onComplete: VoidAction
    public let isAcceptTermsCompleted: Bool

    public init(
        service: any GemWalletServiceProtocol,
        biometryService: any BiometryAuthenticatable,
        preferences: ObservablePreferences,
        nameService: any GemNameServiceProtocol,
        onComplete: VoidAction,
    ) {
        self.service = service
        self.biometryService = biometryService
        self.preferences = preferences
        self.nameService = nameService
        self.onComplete = onComplete
        isAcceptTermsCompleted = preferences.isAcceptTermsCompleted
    }

    var shouldOfferAuthentication: Bool {
        preferences.shouldOfferAuthentication(service: biometryService)
    }

    func importWalletModel(type: ImportWalletType, onComplete: VoidAction) -> ImportWalletSceneViewModel {
        ImportWalletSceneViewModel(service: service, nameService: nameService, type: type, onComplete: onComplete)
    }

    func enableAuthenticationModel() -> EnableAuthenticationSceneViewModel {
        EnableAuthenticationSceneViewModel(service: biometryService, preferences: preferences, onComplete: onComplete)
    }

    func importWalletTypeModel() -> ImportWalletTypeSceneViewModel {
        ImportWalletTypeSceneViewModel()
    }
}
