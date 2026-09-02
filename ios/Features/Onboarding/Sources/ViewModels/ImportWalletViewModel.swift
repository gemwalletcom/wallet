// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemChainService
import protocol Gemstone.GemOnboardingServiceProtocol
import GemstonePrimitives
import Preferences
import Foundation
import Primitives
import PrimitivesComponents
import SwiftUI

@Observable
@MainActor
public final class ImportWalletViewModel {
    private let service: any GemOnboardingServiceProtocol & AddressInputResolving
    private let preferences: ObservablePreferences
    private let walletImage: @MainActor (Wallet) -> WalletImageViewModel
    let onComplete: VoidAction

    var isPresentingSelectImageWallet: Wallet?

    public init(
        service: any GemOnboardingServiceProtocol & AddressInputResolving,
        preferences: ObservablePreferences,
        walletImage: @escaping @MainActor (Wallet) -> WalletImageViewModel,
        onComplete: VoidAction,
    ) {
        self.service = service
        self.preferences = preferences
        self.walletImage = walletImage
        self.onComplete = onComplete
    }

    public var isAcceptTermsCompleted: Bool {
        preferences.isAcceptTermsCompleted
    }

    func importWalletModel(type: ImportWalletType, onComplete: @escaping @MainActor @Sendable (ImportWalletSceneResult) -> Void) -> ImportWalletSceneViewModel {
        ImportWalletSceneViewModel(service: service, preferences: preferences, nameService: service, type: type, onComplete: onComplete)
    }

    func importWalletTypeModel() -> ImportWalletTypeViewModel {
        ImportWalletTypeViewModel(preferences: preferences, chainService: GemChainService())
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
        walletImage(wallet)
    }
}

// MARK: - Actions

extension ImportWalletViewModel {
    func presentSelectImage(wallet: Wallet) {
        isPresentingSelectImageWallet = wallet
    }
}
