// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import Style
import SwiftUI

public struct SwapAmountField {
    public let assetId: AssetId
    public let assetImage: AssetImage
    public let amount: String
    public let fiatAmount: String?

    public init(
        assetId: AssetId,
        assetImage: AssetImage,
        amount: String,
        fiatAmount: String?,
    ) {
        self.assetId = assetId
        self.assetImage = assetImage
        self.amount = amount
        self.fiatAmount = fiatAmount
    }
}

struct SwapAmountView: View {
    let from: SwapAmountField
    let to: SwapAmountField
    let action: TransactionHeaderActionHandler?
    let hideBalance: Bool

    var body: some View {
        VStack(spacing: 0) {
            SwapAmountSingleView(field: from, action: action, hideBalance: hideBalance)
            Images.Actions.receive
                .resizable()
                .colorMultiply(Colors.black)
                .frame(width: 18, height: 22)
                .scaledToFit()
                .padding(.bottom, .small)
                .offset(y: -.small)
            SwapAmountSingleView(field: to, action: action, hideBalance: hideBalance)
        }
    }
}

struct SwapAmountSingleView: View {
    let field: SwapAmountField
    let action: TransactionHeaderActionHandler?
    let hideBalance: Bool

    var body: some View {
        HStack(spacing: 0) {
            if let action {
                Button { action(.header) } label: { textContent }
                    .buttonStyle(.plain)
            } else {
                textContent
            }
            Spacer(minLength: .extraLarge)
            if let action {
                Button { action(.asset(field.assetId)) } label: { AssetImageView(assetImage: field.assetImage) }
                    .buttonStyle(.plain)
            } else {
                AssetImageView(assetImage: field.assetImage)
            }
        }
    }

    private var textContent: some View {
        VStack(alignment: .leading) {
            Text(field.amount.masked(if: hideBalance))
                .foregroundStyle(Colors.black)
                .font(.app.title2)
                .truncationMode(.middle)
                .lineLimit(1)
            if let fiatAmount = field.fiatAmount {
                Text(fiatAmount.masked(if: hideBalance))
                    .font(.app.footnote)
                    .foregroundStyle(Colors.gray)
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .contentShape(Rectangle())
    }
}
