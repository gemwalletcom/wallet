// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public enum URLAction: Equatable {
    case deeplink(DeepLink)
    case walletConnect(WalletConnectAction)
}

public enum WalletConnectAction: Equatable {
    case connect(uri: String)
    case request
    case session(String)
}
