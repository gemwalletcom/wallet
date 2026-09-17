// Copyright (c). Gem Wallet. All rights reserved.

import Components
import PrimitivesComponents
import SwiftUI

public struct ServiceStatusScene: View {
    @Environment(\.isStreamConnected) private var isStreamConnected

    @State private var model: ServiceStatusViewModel

    public init(model: ServiceStatusViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        List {
            Section {
                ForEach(model.itemModels) { item in
                    ListItemView(model: item.listItem)
                }

                ListItemView(model: streamModel.listItem)
            }
        }
        .listRowInsets(.assetListRowInsets)
        .listSectionSpacing(.compact)
        .contentMargins(.top, .scene.top, for: .scrollContent)
        .refreshable {
            await model.load()
        }
        .taskOnce {
            Task { await model.load() }
        }
        .navigationTitle(model.title)
        .navigationBarTitleDisplayMode(.inline)
    }

    private var streamModel: StreamStatusItemViewModel {
        StreamStatusItemViewModel(isConnected: isStreamConnected)
    }
}
