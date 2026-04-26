// Copyright (c). Gem Wallet. All rights reserved.

import Style
import SwiftUI

public struct VerifiedBadgeView: View {
    private let font: Font

    public init(font: Font = .callout) {
        self.font = font
    }

    public var body: some View {
        Images.System.checkmarkSealFill
            .font(font)
            .foregroundStyle(Colors.whiteSolid, Colors.blue)
    }
}
