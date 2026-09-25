// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives

public enum SelectAssetRoute: Hashable {
    case asset(SelectAssetInput)
    case transfer(TransferRoute)
}
