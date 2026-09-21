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
        ListSectionView(provider: model) { row in
            GemListRowView(row: row)
        }
        .listRowInsets(.assetListRowInsets)
        .listSectionSpacing(.compact)
        .contentMargins(.top, .scene.top, for: .scrollContent)
        .refreshable {
            await model.load()
        }
        .task(id: isStreamConnected) {
            await model.load()
        }
        .navigationTitle(model.title)
        .navigationBarTitleDisplayMode(.inline)
    }
}
