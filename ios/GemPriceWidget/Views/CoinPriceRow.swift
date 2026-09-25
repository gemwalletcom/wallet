// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Style
import SwiftUI

struct CoinPriceRow: View {
    private let coin: CoinPrice

    init(coin: CoinPrice) {
        self.coin = coin
    }

    var body: some View {
        HStack(spacing: Spacing.small) {
            AssetImageView(assetImage: coin.assetImage, size: .list.assets.widget)

            VStack(alignment: .leading, spacing: Spacing.extraSmall) {
                Text(coin.name)
                    .font(.app.Widget.callout)
                    .foregroundColor(Colors.black)

                Text(coin.symbol)
                    .font(.caption)
                    .foregroundColor(Colors.secondaryText)
            }

            Spacer()

            VStack(alignment: .trailing, spacing: Spacing.extraSmall) {
                Text(coin.priceText)
                    .font(.app.Widget.callout)
                    .foregroundColor(Colors.black)

                Text(coin.changeText)
                    .font(.caption)
                    .foregroundColor(coin.changeTone.color)
            }
        }
        .padding(.vertical, Spacing.tiny)
    }
}
