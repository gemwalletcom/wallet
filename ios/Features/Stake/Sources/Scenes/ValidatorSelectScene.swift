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
        let options = model.options
        return List {
            ForEach(model.list(options)) { section in
                Section {
                    ForEach(section.values) { row in
                        ValidatorSelectionView(row: row, isSelected: model.isSelected(row)) {
                            model.onSelect(row)
                            dismiss()
                        }
                        .ifLet(model.explorerContext(for: row)) { view, explorerContext in
                            view.explorerContext(explorerContext)
                        }
                    }
                } header: {
                    if let title = section.title {
                        Text(title)
                    }
                }
            }
        }
        .overlay {
            if case let .empty(state) = options.phase {
                EmptyContentView(model: EmptyStateViewModel(state: state))
            }
        }
        .bindQuery(model.validatorsQuery)
        .navigationTitle(model.title)
    }
}
