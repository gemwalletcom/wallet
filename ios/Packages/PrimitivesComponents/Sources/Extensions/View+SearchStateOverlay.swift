// Copyright (c). Gem Wallet. All rights reserved.

import Components
import SwiftUI

public extension View {
    func searchStateOverlay(isLoading: Bool, isEmpty: Bool, empty: EmptyContentType) -> some View {
        overlay {
            if isLoading {
                LoadingView()
            } else if isEmpty {
                EmptyContentView(model: EmptyContentTypeViewModel(type: empty))
            }
        }
    }
}
