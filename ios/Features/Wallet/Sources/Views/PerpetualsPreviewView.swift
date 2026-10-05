// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Perpetuals
import Primitives
import Store
import Style
import SwiftUI

struct PerpetualsPreviewView: View {
    @State private var viewModel: PerpetualsPreviewViewModel
    @Binding private var showBalancePrivacy: Bool

    private let wallet: Wallet

    init(
        wallet: Wallet,
        showBalancePrivacy: Binding<Bool>,
    ) {
        self.wallet = wallet
        _showBalancePrivacy = showBalancePrivacy
        _viewModel = State(initialValue: PerpetualsPreviewViewModel(walletId: wallet.id))
    }

    var body: some View {
        Group {
            switch viewModel.preview {
            case let .trade(balance):
                NavigationLink(value: Scenes.Perpetuals()) {
                    tradePerpetualsItem(balance: balance.text())
                }
            case .positions:
                PerpetualPositionsList(
                    positions: viewModel.positions,
                    showBalancePrivacy: $showBalancePrivacy,
                )
            }
        }
        .bindQuery(viewModel.positionsQuery, viewModel.walletBalanceQuery)
        .onChange(of: wallet.id) { _, newWalletId in
            viewModel.updateWallet(walletId: newWalletId)
        }
    }

    private func tradePerpetualsItem(balance: String) -> some View {
        HStack {
            Text(viewModel.tradePerpetualsTitle)
                .textStyle(ListItemModel.StyleDefaults.titleStyle)
                .lineLimit(1)
                .truncationMode(.tail)

            Spacer(minLength: .extraSmall)

            PrivacyText(balance, isEnabled: $showBalancePrivacy)
                .textStyle(ListItemModel.StyleDefaults.subtitleStyle)
                .multilineTextAlignment(.trailing)
                .lineLimit(1)
                .truncationMode(.middle)
        }
    }
}
