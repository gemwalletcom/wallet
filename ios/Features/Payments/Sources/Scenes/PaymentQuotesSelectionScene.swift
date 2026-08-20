// Copyright (c). Gem Wallet. All rights reserved.

import Components
import PrimitivesComponents
import SwiftUI

public struct PaymentQuotesSelectionScene: View {
    private let model: PaymentSceneViewModel

    public init(model: PaymentSceneViewModel) {
        self.model = model
    }

    public var body: some View {
        SelectableListNavigationStack(
            model: model.quotesModel,
            onFinishSelection: model.onFinishQuotesSelection,
            listContent: { ListAssetItemView(model: $0) },
        )
    }
}
