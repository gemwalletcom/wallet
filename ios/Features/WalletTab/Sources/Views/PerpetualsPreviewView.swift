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
            if viewModel.hasNoPositions {
                NavigationLink(value: Scenes.Perpetuals()) {
                    tradePerpetualsItem
                }
            } else {
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

    private var tradePerpetualsItem: some View {
        HStack {
            Text("Trade Perpetuals")
                .textStyle(ListItemModel.StyleDefaults.titleStyle)
                .lineLimit(1)
                .truncationMode(.tail)

            Spacer(minLength: .extraSmall)

            PrivacyText(viewModel.tradePerpetualsSubtitle, isEnabled: $showBalancePrivacy)
                .textStyle(ListItemModel.StyleDefaults.subtitleStyle)
                .multilineTextAlignment(.trailing)
                .lineLimit(1)
                .truncationMode(.middle)
        }
    }
}
