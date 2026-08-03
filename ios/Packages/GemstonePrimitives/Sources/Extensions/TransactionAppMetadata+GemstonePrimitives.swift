// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension GemTransactionAppMetadata {
    func map() -> TransactionAppMetadata {
        TransactionAppMetadata(name: name, description: description, url: url, icon: icon)
    }
}

public extension TransactionAppMetadata {
    func map() -> GemTransactionAppMetadata {
        GemTransactionAppMetadata(name: name, description: description, url: url, icon: icon)
    }

    var shortName: String {
        Gemstone.walletConnectAppShortName(name: name)
    }
}

public extension TransactionAppMetadata {
    init(merchant: PaymentMerchant) {
        self.init(name: merchant.name, description: .none, url: Gemstone.paymentWalletConnectUrl(), icon: merchant.iconUrl)
    }
}
