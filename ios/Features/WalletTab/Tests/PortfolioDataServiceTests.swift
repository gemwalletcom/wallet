// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitivesTestKit
import PriceService
import PriceServiceTestKit
import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing
@testable import WalletTab
import WalletTabTestKit

struct PortfolioDataServiceTests {
    @Test
    func walletAllTimeValuesConvertedByCurrencyRate() async throws {
        let db = DB.mock()
        let rate: Float = 3.67
        let date = Date(timeIntervalSince1970: 0)
        let currency = Currency.eur.rawValue
        try FiatRateStore(db: db).add([FiatRate(symbol: .eur, rate: Double(rate))])

        let service = PortfolioDataService.mock(
            portfolioService: .mock(apiService: GemPortfolioServiceMock(
                allTimeHigh: .mock(date: date, value: 100),
                allTimeLow: .mock(date: date, value: 20),
            )),
            priceService: .mock(db: db),
        )

        let data = try await service.getPortfoliData(input: .wallet(walletId: .mock(), period: .all, currencyCode: currency))

        let expected: [PortfolioStatistic] = [
            .allTimeHigh(.mock(date: date, value: 100 * rate)),
            .allTimeLow(.mock(date: date, value: 20 * rate)),
        ]
        #expect(data.statistics == expected)
    }
}
