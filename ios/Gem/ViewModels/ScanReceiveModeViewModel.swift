// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import PrimitivesComponents

struct ScanReceiveModeViewModel: Identifiable {
    let mode: ScanReceiveMode

    var id: ScanReceiveMode { mode }

    var title: String {
        mode.title
    }
}
