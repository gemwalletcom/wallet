// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
import Primitives
import Testing

struct PerpetualPriceFormatterTests {
    let formatter = PerpetualFormatter(provider: PerpetualProvider.hypercore)

    @Test
    func formatInputPricePassesTheLocaleSeparator() {
        #expect(formatter.formatInputPrice(3397.10, decimals: 0, locale: Locale(identifier: "en_US")) == "3397.1")
        #expect(formatter.formatInputPrice(3397.10, decimals: 0, locale: Locale(identifier: "de_DE")) == "3397,1")
    }
}
