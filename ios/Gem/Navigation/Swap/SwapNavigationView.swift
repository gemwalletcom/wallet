// Copyright (c). Gem Wallet. All rights reserved.

import Components
import InfoSheet
import Primitives
import Style
import Swap
import SwiftUI

struct SwapNavigationView: View {
    @Environment(\.viewModelFactory) private var viewModelFactory

    @State private var model: SwapSceneViewModel

    init(model: SwapSceneViewModel) {
        _model = State(initialValue: model)
    }

    var body: some View {
        SwapScene(model: model)
            .sheet(item: $model.isPresentingInfoSheet) {
                switch $0 {
                case let .info(type):
                    InfoSheetScene(type: type)
                case let .selectAsset(type):
                    SelectAssetSceneNavigationStack(
                        model: viewModelFactory.selectAssetScene(
                            wallet: model.wallet,
                            selectType: .swap(type),
                            selectAssetAction: model.onFinishAssetSelection,
                        ),
                    )
                case .swapDetails:
                    if let model = model.swapDetailsViewModel {
                        NavigationStack {
                            SwapDetailsView(model: Bindable(model))
                                .presentationDetentsForCurrentDeviceSize(expandable: true)
                                .presentationBackground(Colors.grayBackground)
                        }
                    }
                }
            }
    }
}
