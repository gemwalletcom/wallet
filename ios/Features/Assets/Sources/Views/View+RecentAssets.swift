// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import SwiftUI

public extension View {
    @MainActor
    func recentAssetsSheet(
        model: RecentAssetsViewModel,
        onSelect: @escaping (Asset) -> Void,
    ) -> some View {
        @Bindable var model = model
        return sheet(isPresented: $model.isPresenting) {
            RecentsScene(model: model.recentModel(onSelect: onSelect))
        }
    }
}
