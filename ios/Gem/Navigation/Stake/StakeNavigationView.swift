// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import InfoSheet
import Stake
import SwiftUI

struct StakeNavigationView: View {
    @State private var model: StakeSceneViewModel

    init(model: StakeSceneViewModel) {
        _model = State(initialValue: model)
    }

    var body: some View {
        StakeScene(
            model: model,
        )
        .bindQuery(model.delegationsQuery, model.assetQuery, model.validatorsQuery)
        .sheet(item: $model.isPresentingInfoSheet) {
            InfoSheetScene(sheet: $0)
        }
    }
}
