// Copyright (c). Gem Wallet. All rights reserved.

import Components
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import SwiftUI
import Transfer
import struct Gemstone.GemTransferData

struct PerpetualPositionNavigationStack: View {
    @Environment(\.viewModelFactory) private var viewModelFactory

    @State private var navigationPath = NavigationPath()

    let perpetualRecipientData: PerpetualRecipientData
    let wallet: Wallet
    let onComplete: VoidAction

    init(
        perpetualRecipientData: PerpetualRecipientData,
        wallet: Wallet,
        onComplete: VoidAction,
    ) {
        self.perpetualRecipientData = perpetualRecipientData
        self.wallet = wallet
        self.onComplete = onComplete
    }

    var body: some View {
        NavigationStack(path: $navigationPath) {
            AmountNavigationView(
                model: viewModelFactory.amountScene(
                    input: AmountInput(
                        type: .perpetual(perpetualRecipientData),
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
