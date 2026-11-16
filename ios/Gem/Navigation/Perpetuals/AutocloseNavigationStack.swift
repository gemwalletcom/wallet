// Copyright (c). Gem Wallet. All rights reserved.

import Components
import GemstonePrimitives
import Perpetuals
import Primitives
import SwiftUI

struct AutocloseNavigationStack: View {
    @State private var navigationPath = NavigationPath()

    let position: PerpetualPositionData
    let wallet: Wallet
    let onComplete: VoidAction

    var body: some View {
        NavigationStack(path: $navigationPath) {
            AutocloseScene(
                model: AutocloseSceneViewModel(
                    type: .modify(position, onTransferAction: { navigationPath.append(ConfirmTransferInput(data: $0)) }),
                ),
            )
            .navigationDestination(keyedBy: ConfirmTransferInput.self) {
                ConfirmTransferDestination(wallet: wallet, data: $0.data, onComplete: onComplete)
            }
        }
    }
}
