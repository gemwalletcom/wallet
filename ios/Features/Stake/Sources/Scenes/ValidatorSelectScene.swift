// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI

public struct ValidatorSelectScene: View {
    @Environment(\.dismiss) private var dismiss

    @State private var model: ValidatorSelectSceneViewModel

    public init(model: ValidatorSelectSceneViewModel) {
        _model = State(wrappedValue: model)
    }

    public var body: some View {
        GemGroupListView(list: model.options) { item in
            guard case let .validator(row, _) = item else { return }
            model.onSelect(row)
            dismiss()
        }
        .bindQuery(model.validatorsQuery)
        .navigationTitle(model.title)
    }
}
