// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import WidgetKit

struct PriceWidgetView: View {
    private let viewModel: PriceWidgetViewModel

    init(entry: PriceWidgetEntry, widgetFamily: WidgetFamily) {
        viewModel = PriceWidgetViewModel(entry: entry, widgetFamily: widgetFamily)
    }

    var body: some View {
        VStack(spacing: .zero) {
            WidgetContentView(viewModel: viewModel)
        }
        .padding(viewModel.padding)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .containerBackground(for: .widget) {
            Color.clear
        }
    }
}
