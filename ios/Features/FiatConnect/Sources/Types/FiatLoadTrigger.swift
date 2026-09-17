// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemFiatQuoteRequest
import struct Gemstone.GemFiatSession

struct FiatLoadTrigger: DebouncableTrigger {
    let request: GemFiatQuoteRequest
    let isImmediate: Bool

    init?(session: GemFiatSession, isImmediate: Bool) {
        guard let request = session.quoteRequest() else { return nil }
        self.request = request
        self.isImmediate = isImmediate
    }
}
