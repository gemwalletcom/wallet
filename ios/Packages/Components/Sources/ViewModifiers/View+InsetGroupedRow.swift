// Copyright (c). Gem Wallet. All rights reserved.

import Style
import SwiftUI

public extension View {
    func insetGroupedRow(
        padding: CGFloat = .medium,
        cornerRadius: CGFloat = .medium,
    ) -> some View {
        frame(maxWidth: .infinity, alignment: .leading)
            .padding(padding)
            .background(Colors.listStyleColor, in: RoundedRectangle(cornerRadius: cornerRadius))
    }
}
