// Copyright (c). Gem Wallet. All rights reserved.

import Components
import NFT
import Primitives
import Style
import SwiftUI

struct CollectionsSceneNavigationView: View {
    @Environment(\.viewModelFactory) private var viewModelFactory

    @State private var model: CollectionsViewModel

    init(model: CollectionsViewModel) {
        _model = State(initialValue: model)
    }

    var body: some View {
        CollectionsScene(model: model)
            .sheet(item: $model.isPresentingReceiveSelectAssetType) {
                SelectAssetSceneNavigationStack(
                    model: viewModelFactory.selectAssetScene(
                        wallet: model.wallet,
                        selectType: $0,
                    ),
                )
            }
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button(action: model.onSelectReceive) {
                        Images.System.plus
                    }
                }
            }
    }
}
