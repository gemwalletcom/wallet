// Copyright (c). Gem Wallet. All rights reserved.

import Contacts
import Foundation
import GemstonePrimitives
import GemstoneServices
import Onboarding
import Primitives
import PrimitivesComponents
import Store
import SwiftUI
import Wallets

public extension ViewModelFactory {
    @MainActor
    func walletsScene(
        navigationPath: Binding<NavigationPath>,
        isPresentingCreateWalletSheet: Binding<Bool>,
        isPresentingImportWalletSheet: Binding<Bool>,
    ) -> WalletsSceneViewModel {
        WalletsSceneViewModel(
            navigationPath: navigationPath,
            walletService: walletService,
            biometry: biometryService,
            isPresentingCreateWalletSheet: isPresentingCreateWalletSheet,
            isPresentingImportWalletSheet: isPresentingImportWalletSheet,
        )
    }

    @MainActor
    func walletDetailScene(navigationPath: Binding<NavigationPath>, wallet: Wallet) -> WalletDetailSceneViewModel {
        WalletDetailSceneViewModel(
            navigationPath: navigationPath,
            wallet: wallet,
            service: walletService,
            biometry: biometryService,
        )
    }

    @MainActor
    func walletImageScene(wallet: Wallet) -> WalletImageSceneViewModel {
        WalletImageSceneViewModel(wallet: wallet, service: walletService)
    }

    @MainActor
    func createWalletScene(onComplete: VoidAction) -> CreateWalletViewModel {
        CreateWalletViewModel(service: walletService, preferences: observablePreferences, onComplete: onComplete)
    }

    @MainActor
    func importWalletScene(onComplete: VoidAction) -> ImportWalletViewModel {
        ImportWalletViewModel(
            service: walletService,
            preferences: observablePreferences,
            nameService: nameService,
            onComplete: onComplete,
        )
    }

    @MainActor
    func contactsScene(mode: ContactsSceneViewModel.Mode = .list) -> ContactsSceneViewModel {
        ContactsSceneViewModel(service: contactService, contactEditor: contactEditorScene, mode: mode)
    }

    @MainActor
    func contactEditorScene(mode: ContactEditorSceneViewModel.Mode) -> ContactEditorSceneViewModel {
        ContactEditorSceneViewModel(service: contactEditorService, nameService: nameService, mode: mode)
    }
}
