// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemPerpetualPositionAction
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import SwiftUI
import Transfer
import struct Gemstone.GemTransferData

struct PerpetualPositionNavigationStack: View {
    @Environment(\.viewModelFactory) private var viewModelFactory

    @State private var navigationPath = NavigationPath()

    let positionAction: GemPerpetualPositionAction
    let wallet: Wallet
    let onComplete: VoidAction

    init(
        positionAction: GemPerpetualPositionAction,
        wallet: Wallet,
        onComplete: VoidAction,
    ) {
        self.positionAction = positionAction
        self.wallet = wallet
        self.onComplete = onComplete
    }

    var body: some View {
        NavigationStack(path: $navigationPath) {
            AmountNavigationView(
                model: viewModelFactory.amountScene(
                    input: AmountInput(
                        type: .perpetual(positionAction),
                        asset: Chain.hyperCore.defaultAsset(type: .perpetual),
                    ),
                    wallet: wallet,
                    onTransferAction: {
                        navigationPath.append($0)
                    },
                ),
            )
            .toolbar {
                ToolbarDismissItem(
                    type: .close,
                    placement: .topBarLeading,
                )
            }
            .navigationDestination(for: GemTransferData.self) {
                ConfirmTransferNavigationView(
                    model: viewModelFactory.confirmTransferScene(
                        wallet: wallet,
                        data: $0,
                        onComplete: {
                            onComplete?()
                        },
                    ),
                )
            }
        }
    }
}
