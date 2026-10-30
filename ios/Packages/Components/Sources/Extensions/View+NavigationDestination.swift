// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

public extension View {
    func navigationDestination<D: Hashable>(
        keyedBy data: D.Type,
        @ViewBuilder destination: @escaping (D) -> some View,
    ) -> some View {
        navigationDestination(for: data) {
            destination($0)
                .id($0)
        }
    }

    func navigationDestination<D: Hashable>(
        keyedBy data: D.Type,
        in scope: some Hashable,
        @ViewBuilder destination: @escaping (D) -> some View,
    ) -> some View {
        navigationDestination(keyedBy: data) {
            destination($0)
                .id(scope)
        }
    }
}
