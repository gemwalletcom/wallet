// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemNameServiceProtocol
import protocol Gemstone.GemWalletServiceProtocol
import GemstonePrimitives
import GemstoneServices
import Primitives
import PrimitivesComponents
import SwiftUI

@Observable
@MainActor
public final class ImportWalletViewModel {
    private let service: any GemWalletServiceProtocol
    let preferences: ObservablePreferences
    private let nameService: any GemNameServiceProtocol
    let onComplete: VoidAction

    public init(
        service: any GemWalletServiceProtocol,
        preferences: ObservablePreferences,
        nameService: any GemNameServiceProtocol,
        onComplete: VoidAction,
    ) {
        self.service = service
        self.preferences = preferences
        self.nameService = nameService
        self.onComplete = onComplete
    }

    public var isAcceptTermsCompleted: Bool {
        preferences.isAcceptTermsCompleted
    }

    func importWalletModel(type: ImportWalletType) -> ImportWalletSceneViewModel {
        ImportWalletSceneViewModel(service: service, preferences: preferences, nameService: nameService, type: type, onComplete: onComplete)
    }

    func importWalletTypeModel() -> ImportWalletTypeViewModel {
        ImportWalletTypeViewModel()
    }
}
