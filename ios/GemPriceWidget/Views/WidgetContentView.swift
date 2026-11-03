// Copyright (c). Gem Wallet. All rights reserved.

import Style
import SwiftUI
import WidgetKit

struct WidgetContentView: View {
    private let viewModel: PriceWidgetViewModel

    init(viewModel: PriceWidgetViewModel) {
        self.viewModel = viewModel
    }

    var body: some View {
        if viewModel.entry.error != nil {
            WidgetErrorView(error: viewModel.entry.error, showMessage: viewModel.showsErrorMessage)
        } else if !viewModel.prices.isEmpty {
            switch viewModel.widgetFamily {
            case .systemSmall:
                if let bitcoin = viewModel.prices.first {
                    SmallCoinView(coin: bitcoin)
                }
            default:
                ForEach(viewModel.prices) { coin in
                    CoinPriceRow(coin: coin)
                }
            }
        } else {
            WidgetEmptyView(message: viewModel.emptyMessage)
        }
    }
}
