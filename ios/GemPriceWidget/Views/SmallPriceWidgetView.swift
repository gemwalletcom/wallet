// Copyright (c). Gem Wallet. All rights reserved.

import Style
import SwiftUI
import WidgetKit

struct SmallPriceWidgetView: View {
    private let viewModel: PriceWidgetViewModel

    init(viewModel: PriceWidgetViewModel) {
        self.viewModel = viewModel
    }

    var body: some View {
        VStack(spacing: 0) {
            WidgetContentView(viewModel: viewModel, showErrorMessage: false)
        }
        .padding(Spacing.small)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}
