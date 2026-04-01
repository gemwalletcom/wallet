// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Style
import SwiftUI
import WidgetKit

struct PriceWidgetView: View {
    private let viewModel: PriceWidgetViewModel

    init(entry: PriceWidgetEntry, widgetFamily: WidgetFamily) {
        viewModel = PriceWidgetViewModel(entry: entry, widgetFamily: widgetFamily)
    }

    init(viewModel: PriceWidgetViewModel) {
        self.viewModel = viewModel
    }

    var body: some View {
        Group {
            switch viewModel.widgetFamily {
            case .systemSmall:
                SmallPriceWidgetView(viewModel: viewModel)
            default:
                MediumPriceWidgetView(viewModel: viewModel)
            }
        }
        .containerBackground(for: .widget) {
            Color.clear
        }
    }
}
