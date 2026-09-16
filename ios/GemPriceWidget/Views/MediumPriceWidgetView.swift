// Copyright (c). Gem Wallet. All rights reserved.

import Components
import SwiftUI
import WidgetKit

struct MediumPriceWidgetView: View {
    private let viewModel: PriceWidgetViewModel

    init(viewModel: PriceWidgetViewModel) {
        self.viewModel = viewModel
    }

    var body: some View {
        VStack(spacing: .zero) {
            WidgetContentView(viewModel: viewModel)
        }
        .padding(.zero)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}
