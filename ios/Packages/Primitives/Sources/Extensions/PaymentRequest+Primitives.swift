import Foundation

public extension PaymentRequest {
    var exactAmount: String? {
        switch amount {
        case let .exactValue(value): value
        case .atomicValue, .none: .none
        }
    }
}
