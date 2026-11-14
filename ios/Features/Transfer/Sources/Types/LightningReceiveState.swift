// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemLightningReceive

enum LightningReceiveState: Equatable {
    case loading
    case ready(GemLightningReceive)
    case locked
    case unavailable
}
