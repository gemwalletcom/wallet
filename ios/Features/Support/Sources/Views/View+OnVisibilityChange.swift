// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

extension View {
    @ViewBuilder
    func onVisibilityChange(active: Bool, threshold: Double = 0.1, _ action: @escaping (_ isVisible: Bool) -> Void) -> some View {
        if #available(iOS 18, *), active {
            onScrollVisibilityChange(threshold: threshold) { action($0) }
        } else {
            self
        }
    }
}
