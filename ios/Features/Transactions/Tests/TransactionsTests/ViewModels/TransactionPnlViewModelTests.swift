import Primitives
import PrimitivesTestKit
import Style
import Testing
@testable import Transactions

struct TransactionPnlViewModelTests {
    @Test
    func positivePnl() {
        if case let .pnl(_, value, color) = TransactionPnlViewModel(metadata: .mock(pnl: 100)).itemModel {
            #expect(value.contains("+"))
            #expect(color == Colors.green)
        } else {
            Issue.record("Expected pnl item")
        }
    }

    @Test
    func negativePnl() {
        if case let .pnl(_, value, color) = TransactionPnlViewModel(metadata: .mock(pnl: -50)).itemModel {
            #expect(value.contains("-"))
            #expect(color == Colors.red)
        } else {
            Issue.record("Expected pnl item")
        }
    }

    @Test
    func zeroPnl() {
        if case .empty = TransactionPnlViewModel(metadata: .mock(pnl: 0)).itemModel {
        } else {
            Issue.record("Expected empty")
        }
    }

    @Test
    func nonPerpetualMetadata() {
        if case .empty = TransactionPnlViewModel(metadata: nil).itemModel {
        } else {
            Issue.record("Expected empty")
        }
    }
}
