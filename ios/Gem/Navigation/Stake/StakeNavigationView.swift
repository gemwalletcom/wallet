// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
import InfoSheet
import Primitives
import Stake
import SwiftUI
import Transfer

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
        .ifLet(model.stakeInfoUrl, content: { view, url in
            view.toolbarInfoButton(url: url)
        })
        .sheet(item: $model.isPresentingInfoSheet) {
            InfoSheetScene(type: $0)
        }
    }
}
