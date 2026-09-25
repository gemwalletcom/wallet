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
        List {
            ForEach(model.list) { section in
                Section(section.section) {
                    ForEach(section.values) { value in
                        ValidatorSelectionView(row: value.value, isSelected: model.isSelected(value.value)) {
                            model.onSelect(value.value)
                            dismiss()
                        }
                        .ifLet(model.explorerContext(for: value.value)) { view, explorerContext in
                            view.explorerContext(explorerContext)
                        }
                    }
                }
            }
        }
        .overlay {
            if model.list.isEmpty {
                EmptyContentView(model: model.emptyContent)
            }
        }
        .bindQuery(model.validatorsQuery)
        .navigationTitle(model.title)
    }
}
