import GemstonePrimitivesTestKit
import Primitives
import Testing
@testable import Transactions

struct TransactionPriceViewModelTests {
    @Test
    func priceValue() {
        if case let .price(item) = TransactionPriceViewModel(price: .mock(value: 50000, notation: .plain)).itemModel {
            #expect(item.subtitle?.contains("50") == true)
        } else {
            Issue.record("Expected price item")
        }
    }

    @Test
    func noPrice() {
        if case .empty = TransactionPriceViewModel(price: nil).itemModel {
        } else {
            Issue.record("Expected empty")
        }
    }
}
