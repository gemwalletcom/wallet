import BigInt
import Formatters
import Foundation
import Localization
import Primitives

public enum TransferError: Equatable {
    case invalidAmount
    case minimumAmount(asset: Asset, required: BigInt)
    case invalidAddress(asset: Asset)
}

extension TransferError: LocalizedError {
    public var errorDescription: String? {
        switch self {
        case .invalidAmount:
            Localized.Errors.invalidAmount
        case let .minimumAmount(asset, required):
            Localized.Transfer.minimumAmount(ValueFormatter(style: .auto).string(required, asset: asset).boldMarkdown())
        case let .invalidAddress(asset):
            Localized.Errors.invalidAssetAddress(asset.name.boldMarkdown())
        }
    }
}
