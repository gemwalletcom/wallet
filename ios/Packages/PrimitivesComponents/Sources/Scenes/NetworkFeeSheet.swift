// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import SwiftUI

public struct NetworkFeeSheet: View {
    private let model: NetworkFeeSceneViewModel

    public init(model: NetworkFeeSceneViewModel) {
        self.model = model
    }

    public var body: some View {
        NavigationStack {
            NetworkFeeScene(model: model)
        }
        .sheetPresentation(presentationDetent)
    }

    private var presentationDetent: Set<PresentationDetent> {
        if model.showFeeRates || model.showFeeAssets {
            return Set([.medium, .large])
        }
        return Set([.height(180)])
    }
}
