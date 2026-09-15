// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.PaymentLink
import Primitives
import SwiftUI
import Transfer

struct PaymentNavigationStack: View {
    @Environment(\.navigationHandler) private var navigationHandler
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
        case let .verify(url, link):
            PaymentVerificationScene(model: PaymentVerificationSceneViewModel(url: url, onComplete: { onVerified(link) }))
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

    private func onVerified(_ link: PaymentLink) {
        Task { await navigationHandler.handle(.payment(payment: .link(link: link))) }
    }
}
