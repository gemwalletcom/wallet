// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import SwiftUI
import Transfer

struct PaymentNavigationStack: View {
    @Environment(\.navigationPresenter) private var presenter
    @Environment(\.viewModelFactory) private var viewModelFactory

    private let type: PaymentDestination
    private let wallet: Wallet

    init(type: PaymentDestination, wallet: Wallet) {
        self.type = type
        self.wallet = wallet
    }

    var body: some View {
        switch type {
        case let .confirm(transfer):
            ConfirmTransferNavigationStack(wallet: wallet, transferData: transfer, onComplete: onComplete)
        case let .verify(verification):
            PaymentVerificationScene(
                model: viewModelFactory.paymentVerificationScene(
                    verification: verification,
                    wallet: wallet,
                    onComplete: { presenter.isPresentingPayment.wrappedValue = $0 },
                ),
            )
        case let .recipient(input):
            SelectedAssetNavigationStack(input: input, wallet: wallet, onComplete: onComplete)
        case let .selectAsset(type, chains):
            SelectAssetSceneNavigationStack(
                model: viewModelFactory.selectAssetScene(wallet: wallet, selectType: type, chains: chains),
            )
        }
    }
}

extension PaymentNavigationStack {
    private func onComplete() {
        presenter.isPresentingPayment.wrappedValue = nil
    }
}
