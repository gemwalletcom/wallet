// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemChainService
import protocol Gemstone.GemAvatarServiceProtocol
import protocol Gemstone.GemNameServiceProtocol
import protocol Gemstone.GemWalletServiceProtocol
import GemstonePrimitives
import Preferences
import Foundation
import Primitives
import PrimitivesComponents
import SwiftUI

@Observable
@MainActor
public final class ImportWalletViewModel {
    private let service: any GemWalletServiceProtocol
    private let preferences: ObservablePreferences
    private let nameService: any GemNameServiceProtocol
    private let avatarService: any GemAvatarServiceProtocol
    let onComplete: VoidAction

    var isPresentingSelectImageWallet: Wallet?

    public init(
        service: any GemWalletServiceProtocol,
        preferences: ObservablePreferences,
        nameService: any GemNameServiceProtocol,
        avatarService: any GemAvatarServiceProtocol,
        onComplete: VoidAction,
    ) {
        self.service = service
        self.preferences = preferences
        self.nameService = nameService
        self.avatarService = avatarService
        self.onComplete = onComplete
    }

    public var isAcceptTermsCompleted: Bool {
        preferences.isAcceptTermsCompleted
    }

    func importWalletModel(type: ImportWalletType, onComplete: @escaping @MainActor @Sendable (ImportWalletSceneResult) -> Void) -> ImportWalletSceneViewModel {
        ImportWalletSceneViewModel(service: service, preferences: preferences, nameService: nameService, type: type, onComplete: onComplete)
    }

    func importWalletTypeModel() -> ImportWalletTypeViewModel {
        ImportWalletTypeViewModel(preferences: preferences, service: GemChainService())
    }

    func setupWalletModel(wallet: Wallet, onComplete: @escaping (Wallet) -> Void) -> SetupWalletViewModel {
        SetupWalletViewModel(
            wallet: wallet,
            service: service,
            onSelectImage: { [weak self] in self?.presentSelectImage(wallet: $0) },
            onComplete: onComplete,
        )
    }

    func walletImageModel(wallet: Wallet) -> WalletImageViewModel {
        WalletImageViewModel(wallet: wallet, source: .onboarding, avatarService: avatarService)
    }
}

// MARK: - Actions

extension ImportWalletViewModel {
    func presentSelectImage(wallet: Wallet) {
        isPresentingSelectImageWallet = wallet
    }
}
