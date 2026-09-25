// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import PrimitivesComponents
import SwiftUI

public struct AddressDetailsScene: View {
    @State private var model: AddressDetailsSceneViewModel

    public init(model: AddressDetailsSceneViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        ListSectionView(provider: model) { row in
            GemListRowView(row: row, onCopy: model.onCopy)
        }
        .listSectionSpacing(.compact)
        .contentMargins([.top], .small, for: .scrollContent)
        .navigationTitle(model.title)
        .navigationBarTitleDisplayMode(.inline)
        .bindQuery(model.addressNameQuery)
        .copyToast($model.copyToast)
        .refreshable {
            await model.refresh()
        }
        .taskOnce {
            Task {
                await model.refresh()
            }
        }
    }
}
