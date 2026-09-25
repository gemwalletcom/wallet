// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import Primitives
import PrimitivesComponents
import SwiftUI
import Transfer

struct AmountNavigationStack: View {
    @Environment(\.viewModelFactory) private var viewModelFactory

    @State private var navigationPath = NavigationPath()

    let input: AmountInput
    let wallet: Wallet
    let onComplete: VoidAction

    init(
        input: AmountInput,
        wallet: Wallet,
        onComplete: VoidAction,
    ) {
        self.input = input
        self.wallet = wallet
        self.onComplete = onComplete
    }

    var body: some View {
        NavigationStack(path: $navigationPath) {
            AmountNavigationView(
                model: viewModelFactory.amountScene(
                    input: input,
                    wallet: wallet,
                    onTransferAction: {
                        navigationPath.append(ConfirmTransferInput(data: $0))
                    },
                ),
            )
            .toolbar {
                ToolbarDismissItem(
                    type: .close,
                    placement: .topBarLeading,
                )
            }
            .navigationDestination(for: ConfirmTransferInput.self) {
                ConfirmTransferNavigationView(
                    model: viewModelFactory.confirmTransferScene(
                        wallet: wallet,
                        data: $0.data,
                        onComplete: onComplete,
                    ),
                )
            }
        }
    }
}
