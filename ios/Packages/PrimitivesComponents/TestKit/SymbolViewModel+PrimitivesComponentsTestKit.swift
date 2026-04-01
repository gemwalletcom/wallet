// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
@testable import PrimitivesComponents
import PrimitivesTestKit

public extension SymbolViewModel {
    static func mock(asset: Asset = .mock()) -> SymbolViewModel {
        SymbolViewModel(asset: asset)
    }
}
