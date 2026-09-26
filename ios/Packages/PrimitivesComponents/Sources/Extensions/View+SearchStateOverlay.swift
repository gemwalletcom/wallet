// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Style
import SwiftUI

public enum SearchContentState {
    case loading
    case empty(EmptyStateViewModel)
    case results
}

public extension View {
    func searchStateOverlay(_ state: SearchContentState, background: Color) -> some View {
        overlay {
            switch state {
            case .loading:
                searchStateContent(background: background) {
                    LoadingView()
                }
            case let .empty(model):
                searchStateContent(background: background) {
                    EmptyContentView(model: model)
                }
            case .results:
                EmptyView()
            }
        }
    }
}

private func searchStateContent(background: Color, @ViewBuilder content: () -> some View) -> some View {
    content()
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(background)
}
