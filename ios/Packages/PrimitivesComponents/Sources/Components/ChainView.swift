// Copyright (c). Gem Wallet. All rights reserved.

import Components
import SwiftUI

public struct ChainView: View {
    private let model: any SimpleListItemViewable

    public init(model: any SimpleListItemViewable) {
        self.model = model
    }

    public var body: some View {
        SimpleListItemView(model: model)
    }
}

// MARK: - Previews

#Preview {
    ChainView(model: ChainViewModel(chain: .aptos))
}
