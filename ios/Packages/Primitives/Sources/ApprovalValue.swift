import BigInt
import Foundation

public enum ApprovalValue: Sendable, Equatable, Hashable {
    case exact(BigInt)
    case unlimited

    public init?(value: String?, isUnlimited: Bool) {
        if isUnlimited {
            self = .unlimited
        } else {
            guard let value, let exact = BigInt(value, radix: 10) else {
                return nil
            }
            self = .exact(exact)
        }
    }
}
