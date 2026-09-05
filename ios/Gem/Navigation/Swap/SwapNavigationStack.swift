// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemTransferData
import Primitives
import Swap
import SwiftUI
import Transfer

struct SwapNavigationStack: View {
    @Environment(\.viewModelFactory) private var viewModelFactory
    @State private var navigationPath = NavigationPath()

    private let wallet: Wallet
    private let onComplete: VoidAction

    init(wallet: Wallet, onComplete: VoidAction) {
        self.wallet = wallet
        self.onComplete = onComplete
    }

    var body: some View {
        NavigationStack(path: $navigationPath) {
            SwapNavigationView(
                model: viewModelFactory.swapScene(
                    input: SwapInput(
                        wallet: wallet,
                        pairSelector: SwapPairSelectorViewModel(fromAssetId: nil, toAssetId: nil),
                    ),
                    onSwap: { navigationPath.append($0) },
                ),
            )
            .toolbarDismissItem(type: .close, placement: .topBarLeading)
            .navigationBarTitleDisplayMode(.inline)
            .navigationDestination(for: GemTransferData.self) { data in
                ConfirmTransferNavigationView(
                    model: viewModelFactory.confirmTransferScene(
                        wallet: wallet,
                        data: data,
                        onComplete: onComplete,
                    ),
                )
            }
        }
    }
}
