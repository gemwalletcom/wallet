// Copyright (c). Gem Wallet. All rights reserved.

@testable import FiatConnect
import Foundation
import struct Gemstone.GemFiatQuoteRow
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesTestKit

extension FiatQuoteViewModel {
    static func mock(
        row: GemFiatQuoteRow = .mock(),
        locale: Locale = .US,
    ) -> FiatQuoteViewModel {
        FiatQuoteViewModel(asset: .mock(), row: row, locale: locale)
    }
}
