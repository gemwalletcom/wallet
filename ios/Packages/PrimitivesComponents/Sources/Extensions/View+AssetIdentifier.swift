// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import SwiftUI

public extension View {
    func assetIdentifier(_ assetId: AssetId) -> some View {
        accessibilityElement(children: .contain)
            .accessibilityIdentifier(assetId.identifier)
    }
}
