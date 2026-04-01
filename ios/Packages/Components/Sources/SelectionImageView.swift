// Copyright (c). Gem Wallet. All rights reserved.

import Style
import SwiftUI

public struct SelectionImageView: View {
    public init() {}

    public var body: some View {
        Images.Wallets.selected
            .resizable()
            .scaledToFit()
            .frame(
                width: .list.selected.image,
                height: .list.selected.image,
            )
    }
}

// MARK: - Previews

#Preview {
    SelectionImageView()
}
