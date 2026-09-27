// Copyright (c). Gem Wallet. All rights reserved.

import Components
import NFT
import Primitives
import Style
import SwiftUI

struct CollectionsNavigationView: View {
    @Environment(\.viewModelFactory) private var viewModelFactory

    @State private var model: CollectionsSceneViewModel

    init(model: CollectionsSceneViewModel) {
        _model = State(initialValue: model)
    }

    var body: some View {
        CollectionsScene(model: model)
            .sheet(item: $model.isPresentingReceiveSelectAssetType) {
                SelectAssetNavigationStack(
                    model: viewModelFactory.selectAssetScene(
                        wallet: model.wallet,
                        selectType: $0,
                    ),
                )
            }
            .toolbar {
                if model.screen.offersReceive {
                    ToolbarItem(placement: .navigationBarTrailing) {
                        Button(action: model.onSelectReceive) {
                            Images.System.plus
                        }
                    }
                }
            }
    }
}
