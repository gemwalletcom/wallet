// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
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
    @State private var isVisible = false
    @State private var isDismissed = false
    @State private var bannerHeight: CGFloat = .zero

    private var model: ConnectionStatusViewModel {
        ConnectionStatusViewModel(status: connectionStatus)
    }

    func body(content: Content) -> some View {
        let model = model
        let isPresented = isVisible && !isDismissed
        return content
            .contentMargins(.bottom, isPresented ? bannerHeight + .small : nil, for: .scrollContent)
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
            .task(id: model.isVisible) {
                let isVisible = model.isVisible
                guard await (try? Task.sleep(for: ConnectionStatusViewModel.bannerSettleDelay)) != nil else { return }
                self.isVisible = isVisible
                if !isVisible {
                    isDismissed = false
                }
            }
    }
}
