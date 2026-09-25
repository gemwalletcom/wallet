// Copyright (c). Gem Wallet. All rights reserved.

import Components
import SwiftUI

public struct ChainView: View {
    private let model: ChainViewModel

    public init(model: ChainViewModel) {
        self.model = model
    }

    public var body: some View {
        ListItemView(model: model.listItem)
    }
}

// MARK: - Previews

#Preview {
    ChainView(model: ChainViewModel(chain: .aptos))
}
