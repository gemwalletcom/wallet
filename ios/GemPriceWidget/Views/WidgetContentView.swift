// Copyright (c). Gem Wallet. All rights reserved.

import Style
import SwiftUI
import WidgetKit

struct WidgetContentView: View {
    private let viewModel: PriceWidgetViewModel
    private let showErrorMessage: Bool

    init(viewModel: PriceWidgetViewModel, showErrorMessage: Bool = true) {
        self.viewModel = viewModel
        self.showErrorMessage = showErrorMessage
    }

    var body: some View {
        if viewModel.entry.error != nil {
            WidgetErrorView(error: viewModel.entry.error, showMessage: showErrorMessage)
        } else if !viewModel.prices.isEmpty {
            switch viewModel.widgetFamily {
            case .systemSmall:
                if let bitcoin = viewModel.prices.first {
                    SmallCoinView(
                        model: CoinPriceRowViewModel(coin: bitcoin),
                    )
                }
            default:
                ForEach(viewModel.prices) { coin in
                    CoinPriceRow(
                        model: CoinPriceRowViewModel(coin: coin),
                    )
                }
            }
        } else {
            WidgetEmptyView(message: viewModel.emptyMessage)
        }
    }
}
