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
    var isQuote: Bool = false
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

    @ViewBuilder
    private var inputView: some View {
        if isQuote {
            ZStack(alignment: .leading) {
                Text(text.isEmpty ? side.amountPlaceholder : text)
                    .foregroundStyle(text.isEmpty ? Color(.placeholderText) : Colors.black)
                    .font(.app.title1)
                    .lineLimit(1, reservesSpace: true)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .quoteTransition(value: text, isLoading: showLoading)
                if showLoading {
                    LoadingView()
                }
            }
        } else {
            TextField(side.amountPlaceholder, text: $text)
                .keyboardType(.decimalPad)
                .foregroundStyle(Colors.black)
                .font(.app.title1)
                .disabled(!side.interaction.isAmountEditable)
                .multilineTextAlignment(.leading)
        }
    }

    @ViewBuilder
    private var fiatBalanceView: some View {
        let fiat = Text(side.fiat?.text() ?? " ")
            .lineLimit(1, reservesSpace: true)
            .font(.app.callout)
            .foregroundStyle(Colors.secondaryText)
        if isQuote {
            fiat.quoteTransition(value: side.fiat?.text() ?? "", isLoading: showLoading)
        } else {
            fiat
        }
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

private extension View {
    func quoteTransition(value: String, isLoading: Bool) -> some View {
        contentTransition(.numericText(countsDown: true))
            .animation(value.isEmpty ? nil : .default, value: value)
            .id(isLoading)
            .transition(.asymmetric(insertion: .opacity.animation(.easeIn(duration: .AnimationDuration.normal)), removal: .identity))
    }
}
