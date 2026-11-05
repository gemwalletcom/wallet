// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.LightningInvoice

extension LightningInvoice {
    var expiryDate: Date {
        Date(timeIntervalSince1970: TimeInterval(expiresAt))
    }
}
