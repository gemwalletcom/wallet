// Copyright (c). Gem Wallet. All rights reserved.

import Style
import SwiftUI

public struct IndicatorButton: View {
    private let systemImage: String
    private let showsIndicator: Bool
    private let action: () -> Void

    public init(
        systemImage: String,
        showsIndicator: Bool,
        action: @escaping () -> Void,
    ) {
        self.systemImage = systemImage
        self.showsIndicator = showsIndicator
        self.action = action
    }

    public var body: some View {
        Button(action: action) {
            Image(systemName: systemImage)
        }
        .overlay(alignment: .topTrailing) {
            if showsIndicator {
                indicator
            }
        }
    }

    private var indicator: some View {
        Circle()
            .fill(Colors.blue)
            .frame(width: .small, height: .small)
            .padding(.extraSmall)
            .background(Colors.white)
            .clipShape(Circle())
    }
}
