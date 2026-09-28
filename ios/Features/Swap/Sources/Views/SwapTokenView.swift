// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemSwapSideState
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

struct SwapTokenView: View {
    let side: GemSwapSideState
    @Binding var text: String
    var showLoading: Bool = false
    var onBalanceAction: () -> Void
    var onSelectAssetAction: () -> Void

    var body: some View {
        HStack {
            VStack(alignment: .leading, spacing: .small) {
                inputView
                fiatBalanceView
            }

            VStack(alignment: .trailing, spacing: .small) {
                assetActionView
                availableBalanceView
            }
        }
    }

    private var inputView: some View {
        HStack {
            if showLoading {
                LoadingView()
            }
            TextField(showLoading ? "" : side.amountPlaceholder, text: $text)
                .keyboardType(.decimalPad)
                .foregroundStyle(Colors.black)
                .font(.app.title1)
                .disabled(!side.interaction.isAmountEditable)
                .multilineTextAlignment(.leading)
        }
    }

    private var fiatBalanceView: some View {
        Text(side.fiat?.text() ?? " ")
            .lineLimit(1, reservesSpace: true)
            .font(.app.callout)
            .foregroundStyle(Colors.secondaryText)
    }

    private var assetActionView: some View {
        Button(role: .none) {
            onSelectAssetAction()
        } label: {
            HStack {
                if let icon = side.icon {
                    AssetImageView(assetImage: AssetImage(icon: icon))
                }
                Text(side.title.text)
                    .textStyle(TextStyle(font: .body, color: .primary, fontWeight: .medium))
                    .lineLimit(1)
                SwapChevronView()
            }
            .frame(height: .image.asset)
        }
        .disabled(!side.interaction.isAssetSelectable)
    }

    private var availableBalanceView: some View {
        Button(action: onBalanceAction) {
            Text(side.balance?.text ?? " ")
                .lineLimit(1, reservesSpace: true)
                .font(.app.callout)
                .foregroundStyle(Colors.secondaryText)
        }
        .disabled(!side.interaction.isBalanceActionEnabled)
    }
}
