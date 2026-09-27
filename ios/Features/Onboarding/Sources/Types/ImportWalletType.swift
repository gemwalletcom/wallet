import Foundation
import Primitives

enum ImportWalletType: Hashable {
    case multicoin
    case chain(Chain)
}

extension ImportWalletType {
    var chain: Chain? {
        switch self {
        case .multicoin: .none
        case let .chain(chain): chain
        }
    }
}
