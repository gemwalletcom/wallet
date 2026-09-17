// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemSwapQuoteServiceProtocol
import GemstoneServicesTestKit
import Primitives
import Swap

public extension SwapSlippageViewModel {
    static func mock(
        service: any GemSwapQuoteServiceProtocol = GemSwapQuoteServiceMock(),
        slippage: SwapSlippage = .auto,
        onSelect: @escaping (SwapSlippage) -> Void = { _ in },
    ) -> SwapSlippageViewModel {
        SwapSlippageViewModel(service: service, chain: .ethereum, slippage: slippage, onSelect: onSelect)
    }
}
