// Copyright (c). Gem Wallet. All rights reserved.

import Payments
import Primitives
import SwiftUI

struct PaymentNavigationStack: View {
    @Environment(\.viewModelFactory) private var viewModelFactory

    @State private var navigationPath = NavigationPath()

    let wallet: Wallet
    let link: PaymentLink
    let quotes: PaymentQuotes
    let onComplete: VoidAction

    var body: some View {
        NavigationStack(path: $navigationPath) {
            PaymentNavigationView(
                model: viewModelFactory.paymentScene(
                    wallet: wallet,
                    link: link,
                    quotes: quotes,
                    onTransferAction: { navigationPath.append($0) },
                    onComplete: onComplete,
                ),
            )
            .toolbarDismissItem(type: .close, placement: .topBarLeading)
            .navigationBarTitleDisplayMode(.inline)
            .navigationDestination(for: TransferData.self) { data in
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
