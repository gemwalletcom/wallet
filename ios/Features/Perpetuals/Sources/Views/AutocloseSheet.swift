// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import PrimitivesComponents
import SwiftUI

public struct AutocloseSheet: View {
    @State private var model: AutocloseSceneViewModel

    public init(openData: AutocloseOpenData, onComplete: @escaping AutocloseCompletion) {
        _model = State(initialValue: AutocloseSceneViewModel(type: .open(openData, onComplete: onComplete)))
    }

    public var body: some View {
        NavigationStack {
            AutocloseScene(model: model)
                .navigationBarTitleDisplayMode(.inline)
                .toolbar { ToolbarDismissItem(type: .close, placement: .topBarLeading) }
        }
    }
}
