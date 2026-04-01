// Copyright (c). Gem Wallet. All rights reserved.

import Style
import SwiftUI

public struct VerifiedBadgeView: View {
    public init() {}

    public var body: some View {
        Images.System.checkmarkSealFill
            .font(.callout)
            .foregroundStyle(Colors.whiteSolid, Colors.blue)
    }
}
