// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import Primitives
import SwiftUI
import Wallet

@Observable
final class NavigationPresenter: Sendable {
    @MainActor private var _sheet: NavigationSheetType?

    init() {}
}

@MainActor
extension NavigationPresenter {
    var sheet: Binding<NavigationSheetType?> {
        Binding(get: { self._sheet }, set: { self._sheet = $0 })
    }

    var isPresentingAssetInput: Binding<SelectedAssetInput?> {
        Binding(get: { self._sheet?.assetInput }, set: { self._sheet = $0.map(NavigationSheetType.assetInput) })
    }

    var isPresentingWalletSheet: Binding<WalletSheetType?> {
        Binding(get: { self._sheet?.walletSheet }, set: { self._sheet = $0.map(NavigationSheetType.wallet) })
    }

    var isPresentingWallets: Binding<Bool> {
        Binding(get: { self._sheet?.id == NavigationSheetType.wallets.id }, set: { self._sheet = $0 ? .wallets : nil })
    }

    var isPresentingSupport: Binding<Bool> {
        Binding(get: { self._sheet?.id == NavigationSheetType.support.id }, set: { self._sheet = $0 ? .support : nil })
    }

    func dismiss() {
        _sheet = nil
    }
}
