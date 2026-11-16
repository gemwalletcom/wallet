// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemTransferData
import Primitives
import Style
import SwiftUI

struct ConfirmTransferNavigationStack: View {
    private let wallet: Wallet
    private let transferData: GemTransferData
    private let onComplete: VoidAction

    init(
        wallet: Wallet,
        transferData: GemTransferData,
        onComplete: VoidAction,
    ) {
        self.wallet = wallet
        self.transferData = transferData
        self.onComplete = onComplete
    }

    var body: some View {
        NavigationStack {
            ConfirmTransferDestination(
                wallet: wallet,
                data: transferData,
                onComplete: onComplete,
            )
            .toolbar {
                ToolbarItem(placement: .topBarLeading) {
                    Button("", systemImage: SystemImage.xmark) {
                        onComplete?()
                    }
                }
            }
            .navigationBarTitleDisplayMode(.inline)
            .interactiveDismissDisabled(true)
        }
    }
}
