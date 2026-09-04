import Primitives
import PrimitivesTestKit
import Style
import Testing
@testable import Transactions

struct TransactionPnlViewModelTests {
    @Test
    func positivePnl() {
        if case let .pnl(_, value, color) = TransactionPnlViewModel(pnl: 100).itemModel {
            #expect(value.contains("+"))
            #expect(color == Colors.green)
        } else {
            Issue.record("Expected pnl item")
        }
    }

    @Test
    func negativePnl() {
        if case let .pnl(_, value, color) = TransactionPnlViewModel(pnl: -50).itemModel {
            #expect(value.contains("-"))
            #expect(color == Colors.red)
        } else {
            Issue.record("Expected pnl item")
        }
    }

    @Test
    func noPnl() {
        if case .empty = TransactionPnlViewModel(pnl: nil).itemModel {
        } else {
            Issue.record("Expected empty")
        }
    }
}
