// Copyright (c). Gem Wallet. All rights reserved.

import Components
import func Gemstone.chainRow
import struct Gemstone.GemChainRow
import SwiftUI

public struct ChainView: View {
    private let model: GemChainRow

    public init(model: GemChainRow) {
        self.model = model
    }

    public var body: some View {
        ListItemView(model: model.listItem)
    }
}

// MARK: - Previews

#Preview {
    ChainView(model: chainRow(chain: "aptos"))
}
