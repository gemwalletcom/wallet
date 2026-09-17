// Copyright (c). Gem Wallet. All rights reserved.

import Components
import SwiftUI

public struct AddressDetailsNavigationStack: View {
    private let model: AddressDetailsSceneViewModel

    public init(model: AddressDetailsSceneViewModel) {
        self.model = model
    }

    public var body: some View {
        NavigationStack {
            AddressDetailsScene(model: model)
                .toolbarDismissItem(type: .close, placement: .topBarLeading)
        }
        .sheetPresentation([.large])
    }
}
