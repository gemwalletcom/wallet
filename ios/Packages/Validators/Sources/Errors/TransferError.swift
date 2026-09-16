import Foundation
import Localization
import Primitives

public enum TransferError: Equatable {
    case invalidAmount
    case invalidAddress(asset: Asset)
}

extension TransferError: LocalizedError {
    public var errorDescription: String? {
        switch self {
        case .invalidAmount:
            Localized.Errors.invalidAmount
        case let .invalidAddress(asset):
            Localized.Errors.invalidAssetAddress(asset.name.boldMarkdown())
        }
    }
}
