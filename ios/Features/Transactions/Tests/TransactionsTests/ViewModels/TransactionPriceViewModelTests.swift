import Primitives
import PrimitivesTestKit
import Testing
@testable import Transactions

struct TransactionPriceViewModelTests {
    @Test
    func priceValue() {
        if case let .price(_, value) = TransactionPriceViewModel(price: 50000).itemModel {
            #expect(value.contains("50"))
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
