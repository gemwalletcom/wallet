// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import Perpetuals
import Primitives
import SwiftUI

struct AutocloseNavigationStack: View {
    @Environment(\.viewModelFactory) private var viewModelFactory

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
