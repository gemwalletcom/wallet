// Copyright (c). Gem Wallet. All rights reserved.

import Localization
import Primitives

extension QRScanType {
    var hint: String {
        switch self {
        case .universal: Localized.Wallet.scanHint
        case .walletConnect: Localized.WalletConnect.title
        case .address: Localized.Wallet.scanHintAddress
        case .memo: Localized.Transfer.memo
        case .url: Localized.Common.url
        case .tokenContract: Localized.Wallet.Import.contractAddressField
        case .secretPhrase: Localized.Common.secretPhrase
        case .privateKey: Localized.Common.privateKey
        }
    }
}
