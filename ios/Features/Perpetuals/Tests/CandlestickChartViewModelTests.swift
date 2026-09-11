// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
@testable import Perpetuals
import PerpetualsTestKit
import Primitives
import PrimitivesTestKit
import Testing

struct CandlestickChartViewModelTests {
    @Test
    func labelOffsetsFollowTheOverlapLevels() {
        let model = CandlestickChartViewModel(
            candles: [.mock(high: 200, low: 100)],
            position: .mock(
                entryPrice: 121,
                liquidationPrice: 120,
                takeProfit: PerpetualTriggerOrder(price: 180, order_type: .limit, order_id: "tp"),
            ),
            formatter: CurrencyFormatter(currencyCode: "USD"),
        )

        #expect(model.lines.map(\.price) == [120, 121, 180])
        #expect(model.lineLabelOffsets == [0, CandlestickChartViewModel.Constants.labelOverlapSpacing, 0])
        #expect(model.yAxisRange.contains(100) && model.yAxisRange.contains(200))
    }
}
