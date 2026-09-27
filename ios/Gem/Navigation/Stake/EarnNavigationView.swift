// Copyright (c). Gem Wallet. All rights reserved.

import Stake
import SwiftUI

struct EarnNavigationView: View {
    @State private var model: EarnSceneViewModel

    init(model: EarnSceneViewModel) {
        _model = State(initialValue: model)
    }

    var body: some View {
        EarnScene(model: model)
            .bindQuery(model.assetQuery, model.positionsQuery, model.providersQuery)
    }
}
