// Copyright (c). Gem Wallet. All rights reserved.

import Primitives

enum ScanReceiveSheetType: Identifiable {
    case filter
    case receive(SelectedAssetInput)

    var id: String {
        switch self {
        case .filter: "filter"
        case let .receive(input): "receive-\(input.id)"
        }
    }
}
