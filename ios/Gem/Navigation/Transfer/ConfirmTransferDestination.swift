// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemTransferData
import Primitives
import SwiftUI

struct ConfirmTransferDestination: View {
    @Environment(\.viewModelFactory) private var viewModelFactory

    let wallet: Wallet
    let data: GemTransferData
    let onComplete: VoidAction

    var body: some View {
        ConfirmTransferNavigationView(
            model: viewModelFactory.confirmTransferScene(
                wallet: wallet,
                data: data,
                onComplete: onComplete,
            ),
        )
    }
}
