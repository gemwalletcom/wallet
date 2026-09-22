// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

struct ButtonProgressView: View {
    private static let rotationDuration: TimeInterval = 1
    private static let arc: CGFloat = 0.75

    @State private var isRotating = false

    var body: some View {
        Circle()
            .trim(from: 0, to: Self.arc)
            .stroke(Colors.whiteSolid, style: StrokeStyle(lineWidth: .space2, lineCap: .round))
            .padding(.space1)
            .frame(width: Sizing.image.small, height: Sizing.image.small)
            .animation(.linear(duration: Self.rotationDuration).repeatForever(autoreverses: false)) { content in
                content.rotationEffect(.degrees(isRotating ? 360 : 0))
            }
            .onAppear { isRotating = true }
    }
}
