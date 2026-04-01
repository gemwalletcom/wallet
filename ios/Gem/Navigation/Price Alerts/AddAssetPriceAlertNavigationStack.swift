// Copyright (c). Gem Wallet. All rights reserved.

import Assets
import Foundation
import PriceAlerts
import Primitives
import PrimitivesComponents
import SwiftUI

struct AddAssetPriceAlertsNavigationStack: View {
    @State private var selectAssetModel: SelectAssetViewModel

    init(selectAssetModel: SelectAssetViewModel) {
        self.selectAssetModel = selectAssetModel
    }

    var body: some View {
        NavigationStack {
            SelectAssetScene(
                model: selectAssetModel,
            )
            .toolbar {
                ToolbarDismissItem(type: .close, placement: .topBarLeading)
            }
            .navigationBarTitleDisplayMode(.inline)
        }
    }
}
