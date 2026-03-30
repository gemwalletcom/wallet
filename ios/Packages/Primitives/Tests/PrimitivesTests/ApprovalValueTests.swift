import BigInt
@testable import Primitives
import PrimitivesTestKit
import Testing

struct ApprovalValueTests {
    @Test
    func initValue() {
        #expect(ApprovalValue(value: "0", isUnlimited: true) == .unlimited)
        #expect(ApprovalValue(value: "1000000", isUnlimited: false) == .exact(BigInt(1_000_000)))
        #expect(ApprovalValue(value: "invalid", isUnlimited: false) == nil)
        #expect(ApprovalValue(value: "", isUnlimited: true) == .unlimited)
    }
}
