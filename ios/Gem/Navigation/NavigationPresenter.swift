// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import Primitives
import SwiftUI
import Wallet

@Observable
final class NavigationPresenter: Sendable {
    @MainActor private var presented: PresentedSheet?

    init() {}
}

@MainActor
extension NavigationPresenter {
    var presentedSheet: Binding<PresentedSheet?> {
        Binding(get: { self.presented }, set: { self.presented = $0 })
    }

    var sheet: Binding<NavigationSheetType?> {
        Binding(get: { self.presented?.type }, set: { self.present($0) })
    }

    var isPresentingAssetInput: Binding<SelectedAssetInput?> {
        Binding(get: { self.presented?.type.assetInput }, set: { self.present($0.map(NavigationSheetType.assetInput)) })
    }

    var isPresentingWalletSheet: Binding<WalletSheetType?> {
        Binding(get: { self.presented?.type.walletSheet }, set: { self.present($0.map(NavigationSheetType.wallet)) })
    }

    var isPresentingWallets: Binding<Bool> {
        Binding(get: { self.presented?.type.id == NavigationSheetType.wallets.id }, set: { self.present($0 ? .wallets : nil) })
    }

    var isPresentingSupport: Binding<Bool> {
        Binding(get: { self.presented?.type.id == NavigationSheetType.support.id }, set: { self.present($0 ? .support : nil) })
    }

    func dismiss() {
        presented = nil
    }

    private func present(_ sheet: NavigationSheetType?) {
        presented = sheet.map(PresentedSheet.init(type:))
    }
}
