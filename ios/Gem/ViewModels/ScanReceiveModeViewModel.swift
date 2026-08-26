// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Localization

struct ScanReceiveModeViewModel: Identifiable {
    let mode: ScanReceiveMode

    var id: ScanReceiveMode { mode }

    var title: String {
        switch mode {
        case .scan: Localized.Wallet.scan
        case .receive: Localized.Wallet.receive
        }
    }
}
