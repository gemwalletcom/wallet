// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

private struct ChartZoomModifier: ViewModifier {
    @Binding var isPinching: Bool
    let onZoom: @MainActor (Double) -> Void

    @GestureState private var magnification: CGFloat?

    func body(content: Content) -> some View {
        content
            .simultaneousGesture(
                MagnifyGesture()
                    .updating($magnification) { value, magnification, _ in
                        onZoom(value.magnification / (magnification ?? 1))
                        magnification = value.magnification
                    },
            )
            .onChange(of: magnification == nil) { _, isIdle in
                isPinching = !isIdle
            }
    }
}

// MARK: - View Modifier

public extension View {
    func chartZoom(isPinching: Binding<Bool>, onZoom: @escaping @MainActor (Double) -> Void) -> some View {
        modifier(ChartZoomModifier(isPinching: isPinching, onZoom: onZoom))
    }
}
