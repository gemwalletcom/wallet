// Copyright (c). Gem Wallet. All rights reserved.

import ConnectionStatusService
import PrimitivesComponents
import Style
import SwiftUI

extension View {
    func connectionStatusBanner() -> some View {
        modifier(ConnectionStatusBannerModifier())
    }
}

private struct ConnectionStatusBannerModifier: ViewModifier {
    @Environment(\.connectionStatus) private var connectionStatus
    @State private var isDismissed = false
    @State private var bannerHeight: CGFloat = .zero

    private var model: ConnectionStatusViewModel {
        ConnectionStatusViewModel(status: connectionStatus.status)
    }

    func body(content: Content) -> some View {
        let model = model
        let isPresented = model.isVisible && !isDismissed
        return content
            .contentMargins(.bottom, isPresented ? bannerHeight + .small : .zero, for: .scrollContent)
            .overlay(alignment: .bottom) {
                if isPresented {
                    ConnectionStatusBanner(model: model) {
                        isDismissed = true
                    }
                    .onGeometryChange(for: CGFloat.self, of: { $0.size.height }) {
                        bannerHeight = $0
                    }
                    .padding(.bottom, .space32 + .space32)
                }
            }
            .onChange(of: model.isVisible) { _, isVisible in
                if isVisible {
                    isDismissed = false
                }
            }
    }
}
