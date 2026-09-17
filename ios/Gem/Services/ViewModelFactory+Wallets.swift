// Copyright (c). Gem Wallet. All rights reserved.

import Contacts
import Foundation
import GemstonePrimitives
import GemstoneServices
import ManageWallets
import Onboarding
import Primitives
import PrimitivesComponents
import Store
import SwiftUI

extension ViewModelFactory {
    @MainActor
    public func walletsScene(
        navigationPath: Binding<NavigationPath>,
        isPresentingCreateWalletSheet: Binding<Bool>,
        isPresentingImportWalletSheet: Binding<Bool>,
    ) -> WalletsSceneViewModel {
        WalletsSceneViewModel(
            navigationPath: navigationPath,
            walletService: walletService,
            preferences: observablePreferences,
            isPresentingCreateWalletSheet: isPresentingCreateWalletSheet,
            isPresentingImportWalletSheet: isPresentingImportWalletSheet,
        )
    }

    @MainActor
    public func walletDetailScene(navigationPath: Binding<NavigationPath>, wallet: Wallet) -> WalletDetailViewModel {
        WalletDetailViewModel(
            navigationPath: navigationPath,
            wallet: wallet,
            service: walletService,
            preferences: observablePreferences,
        )
    }

    @MainActor
    public func walletImageScene(wallet: Wallet) -> WalletImageViewModel {
        WalletImageViewModel(wallet: wallet, service: walletService)
    }

    @MainActor
    public func createWalletScene(onComplete: VoidAction) -> CreateWalletModel {
        CreateWalletModel(service: walletService, preferences: observablePreferences, onComplete: onComplete)
    }

    @MainActor
    public func importWalletScene(onComplete: VoidAction) -> ImportWalletViewModel {
        ImportWalletViewModel(
            service: walletService,
            preferences: observablePreferences,
            nameService: nameService,
            onComplete: onComplete,
        )
    }

    @MainActor
    public func contactsScene(mode: ContactsViewModel.Mode = .list) -> ContactsViewModel {
        ContactsViewModel(service: contactService, manageContact: manageContactScene, mode: mode)
    }

    @MainActor
    public func manageContactScene(mode: ManageContactViewModel.Mode) -> ManageContactViewModel {
        ManageContactViewModel(service: manageContactService, nameService: nameService, mode: mode)
    }
}
