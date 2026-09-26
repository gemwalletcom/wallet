// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemSwapRequest
import struct Gemstone.GemSwapSession
import GemstonePrimitivesTestKit
import Localization
import Primitives
import PrimitivesTestKit
import Style
@testable import Swap
@testable import SwapTestKit
import Testing

struct SwapButtonViewModelTests {
    @Test
    func insufficientBalanceNamesTheAssetAndDisablesTheButton() {
        let asset = AssetData.mock(asset: .mock(symbol: "BTC"))
        let viewModel = SwapButtonViewModel.mock(session: .mock(quotes: .mock(quotes: [.mock()]), selectedQuote: .mock(), quotePhase: .ready, input: .mock(request: .mock(value: 2))), availableBalance: 1, fromAsset: asset)

        #expect(viewModel.title == Localized.Transfer.insufficientBalance("BTC"))
        #expect(viewModel.type == ButtonType.primary(.disabled))
    }
}
