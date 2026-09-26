// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import PrimitivesComponents
import SwiftUI

public struct SetPriceAlertNavigationStack: View {
    private let model: SetPriceAlertSceneViewModel

    public init(model: SetPriceAlertSceneViewModel) {
        self.model = model
    }

    public var body: some View {
        NavigationStack {
            SetPriceAlertScene(model: model)
                .toolbarDismissItem(type: .close, placement: .topBarLeading)
        }
    }
}
