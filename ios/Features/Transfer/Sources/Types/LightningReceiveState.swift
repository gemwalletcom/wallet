// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.LightningInvoice

enum LightningReceiveState: Equatable {
    case opening
    case amount
    case invoice(LightningInvoice)
    case expired(LightningInvoice)
    case unlockRequired
    case unavailable
}
