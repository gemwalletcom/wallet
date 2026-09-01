// Copyright (c). Gem Wallet. All rights reserved.

import GemstoneServices
import Foundation
import GemstonePrimitives
import Primitives
import Style
import SwiftUI
import Transfer

struct ConfirmTransferNavigationStack: View {
    @Environment(\.viewModelFactory) private var viewModelFactory

    private let wallet: Wallet
    private let transferData: TransferData
    private let onComplete: VoidAction

    init(
        wallet: Wallet,
        transferData: TransferData,
        onComplete: VoidAction,
    ) {
        self.wallet = wallet
        self.transferData = transferData
        self.onComplete = onComplete
    }

    var body: some View {
        NavigationStack {
            ConfirmTransferNavigationView(
                model: viewModelFactory.confirmTransferScene(
                    wallet: wallet,
                    data: transferData,
                    onComplete: onComplete,
                ),
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
