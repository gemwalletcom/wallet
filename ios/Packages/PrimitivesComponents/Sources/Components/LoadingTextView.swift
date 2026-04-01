// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import SwiftUI

public struct LoadingTextView: View {
    @Binding var isAnimating: Bool

    public init(isAnimating: Binding<Bool>) {
        _isAnimating = isAnimating
    }

    public var body: some View {
        HStack {
            Text(Localized.Common.loading + "...")
            ActivityIndicator(isAnimating: $isAnimating, style: .medium)
        }
    }
}
